use bevy::{camera::visibility::NoFrustumCulling, light::NotShadowCaster, prelude::*};
use chunk_flow_field::{
    map::{EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP, Obstacle},
    types::Cell,
};

use super::Building;
use crate::{
    input::UiInputCapture,
    map::TerrainHeightMap,
    movement::flow_grid::FlowGrid,
    thing::{ThingCatalog, ThingType},
    ui::components::BlockWorldInput,
    unit::{component::{Position, Team}, spatial_grid::SpatialGrid},
    util::{circle_rect_overlap, footprint_center, screen_to_world, snapped_origin},
    visual::{SpriteCatalog, team_color::TeamColorMaterial},
    world3d::spawn_billboard,
};

#[derive(Debug, Clone, Component)]
pub(super) struct PlacementSession {
    thing_type: ThingType,
    size: UVec2,
    ready: bool,
}

#[derive(Debug, Clone, Component)]
struct Ghost;

#[derive(Debug, Clone, Component)]
pub(super) struct CellContainer {
    size: IVec2,
    grid_pos: IVec2,
}

#[derive(Debug, Clone, Component)]
pub(super) struct CellOverlay {
    green: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    valid: bool,
}

#[derive(Debug, Clone, Component)]
pub(super) struct IsPlacementSession;

#[derive(Event)]
pub struct BeginBuildingPlacement(pub ThingType);

pub(super) fn begin_placement(
    event: On<BeginBuildingPlacement>,
    mut commands: Commands,
    catalog: Res<ThingCatalog>,
    sprites: Res<SpriteCatalog>,
    team_materials: Res<Assets<TeamColorMaterial>>,
    flow_grid: Res<FlowGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let building_type = event.0;
    let Some(stats) = catalog.get_info(building_type).and_then(|info| info.building_stats) else {
        warn!("No building configuration for {:?}", event.0);
        return;
    };
    if stats.validate().is_err()
        || stats.size.x as usize > flow_grid.map.width
        || stats.size.y as usize > flow_grid.map.height
    {
        warn!("Building footprint does not fit this map: {:?}", stats.size);
        return;
    }
    let session = commands.spawn((
        PlacementSession {
            thing_type: building_type,
            size: stats.size,
            ready: false,
        },
        Visibility::Hidden,
        BlockWorldInput,
        IsPlacementSession,
        Transform::default(),
    )).id();
    let Some(visual) = sprites.sprites.get(&event.0) else {
        return;
    };
    let (mesh, source_material) = visual.get_mesh_mat(None, Some(Team::Player));
    let Some(source) = team_materials.get(&source_material) else {
        return;
    };
    let ghost_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.45),
        base_color_texture: source.texture.clone(),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    // spawn building ghost ui
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(ghost_material.clone()),
        Transform::from_translation(visual.model_offset),
        Pickable::IGNORE,
        Visibility::Inherited,
        NotShadowCaster,
        Ghost,
        ChildOf(session),
    ));
    let green = materials.add(cell_material(Color::srgba(0.1, 1.0, 0.2, 0.4)));
    let red = materials.add(cell_material(Color::srgba(1.0, 0.1, 0.1, 0.4)));
    let cell_transform = Transform::from_translation(Vec3::new(
        (stats.size.x - 1) as f32 * flow_grid.cell_size * -0.5,
        0.6,
        (stats.size.y - 1) as f32 * flow_grid.cell_size * -0.5,
    ));
    let cell_container = commands.spawn((
        Visibility::Inherited,
        CellContainer { size: stats.size.as_ivec2(), grid_pos: IVec2::ZERO },
        ChildOf(session),
        cell_transform,
    )).id();
    let mut cells = Vec::new();
    let mut cell_meshes = Vec::new();
    for index in 0..stats.size.x * stats.size.y {
        let mesh = meshes.add(Plane3d::default().mesh().size(
            flow_grid.cell_size * 0.96,
            flow_grid.cell_size * 0.96,
        ));
        let cell_pos = IVec2::new(index as i32 % stats.size.x as i32, index as i32 / stats.size.x as i32);
        let transform = Transform::from_translation(Vec3::new(
            cell_pos.x as f32 * flow_grid.cell_size,
            0.0,
            cell_pos.y as f32 * flow_grid.cell_size,
        ));
        cells.push(commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(red.clone()),
            transform,
            Visibility::Inherited,
            Pickable::IGNORE,
            NotShadowCaster,
            // Vertices follow terrain heights after the mesh's initial AABB is computed.
            NoFrustumCulling,
            ChildOf(cell_container),
            CellOverlay { green: green.clone(), red: red.clone(), valid: false },
        )).id());
        cell_meshes.push(mesh);
    }
}

fn cell_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    }
}

pub(super) fn cell_system(
    mut cell: Query<(&CellOverlay, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    cell.iter_mut().for_each(|(overlay, mut material)| {
        material.0 = if overlay.valid { overlay.green.clone() } else { overlay.red.clone() };
    });
}

pub(super) fn cell_validate_system(
    cell_container: Query<(&CellContainer, &Children)>,
    mut cells: Query<&mut CellOverlay>,
    flow_grid: Res<FlowGrid>,
    spatial_grid: Res<SpatialGrid>,
    heights: Res<TerrainHeightMap>,
) {
    let map_cells = spatial_grid.map_size.as_ivec2();
    let cell_size = flow_grid.cell_size;
    let map_max = (map_cells.as_vec2() * cell_size).map(f32::next_down);
    let terrain_size = Vec2::new(
        heights.width.saturating_sub(1) as f32,
        heights.height.saturating_sub(1) as f32,
    ) * heights.sample_spacing;
    cell_container.iter().for_each(|(container, children)| {
        let query_min = (container.grid_pos.as_vec2() * cell_size).clamp(Vec2::ZERO, map_max);
        let query_max = ((container.grid_pos + container.size).as_vec2() * cell_size)
            .clamp(Vec2::ZERO, map_max);
        let Ok(candidates) = spatial_grid.query_entities_rect(
            query_min,
            query_max,
            false,
        ) else {
            warn!("Spatial query failed for footprint at {:?}", container.grid_pos);
            // Confirmation must not reuse valid flags from a previous frame.
            for child in children.iter() {
                if let Ok(mut overlay) = cells.get_mut(child) {
                    overlay.valid = false;
                }
            }
            return;
        };
        children.iter().enumerate().for_each(|(index, child)| {
            let cell_pos = IVec2::new(index as i32 % container.size.x, index as i32 / container.size.x);
            let grid_pos = container.grid_pos + cell_pos;
            if let Ok(mut overlay) = cells.get_mut(child) {
                let cell_min = grid_pos.as_vec2() * cell_size;
                let cell_max = cell_min + Vec2::splat(cell_size);
                if grid_pos.x < 0 || grid_pos.y < 0
                    || grid_pos.x >= map_cells.x || grid_pos.y >= map_cells.y
                    || cell_max.x > terrain_size.x || cell_max.y > terrain_size.y
                {
                    overlay.valid = false;
                    return;
                }
                overlay.valid = flow_grid.map.can_pass(
                    grid_pos.x as usize,
                    grid_pos.y as usize,
                    EDGE_TOP | EDGE_RIGHT | EDGE_BOTTOM | EDGE_LEFT,
                ) && !matches!(flow_grid.map.get_cell(grid_pos.x as usize, grid_pos.y as usize), Cell::Wall)
                    && candidates.iter().all(|unit| {
                        !circle_rect_overlap(unit.pos, unit.radius, cell_min, cell_max)
                    })
            }
        });
    });
}

fn register_footprint(
    flow: &mut FlowGrid,
    origin: IVec2,
    size: UVec2,
) -> anyhow::Result<chunk_flow_field::map::BuildingId> {
    anyhow::ensure!(size.x > 0 && size.y > 0, "Empty footprint");
    anyhow::ensure!(origin.x >= 0 && origin.y >= 0, "Negative footprint origin");
    anyhow::ensure!(origin.x as u64 + size.x as u64 <= flow.map.width as u64
        && origin.y as u64 + size.y as u64 <= flow.map.height as u64,
        "Footprint is outside the map");
    let mut obstacle = Obstacle::new(flow.map.width, flow.map.height);
    for dy in 0..size.y {
        for dx in 0..size.x {
            obstacle.set_cell_blocked(origin.x as usize + dx as usize, origin.y as usize + dy as usize);
        }
    }
    flow.map.add_building(obstacle)
}

pub(super) fn finish_placement(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    query: Query<Entity, (With<BlockWorldInput>, With<IsPlacementSession>, Without<PlacementSession>)>,
) {
    let mouse_busy = [MouseButton::Left, MouseButton::Right]
    .into_iter()
    .any(|button| buttons.pressed(button) || buttons.just_released(button));

    if mouse_busy {
        return;
    }
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub(super) fn update_placement(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<UiInputCapture>,
    window: Single<&Window>,
    camera: Single<(&Camera, &Transform), With<Camera3d>>,
    session: Single<(Entity, &mut Transform, &mut PlacementSession), Without<Camera3d>>,
    mut container: Single<&mut CellContainer>,
    flow: Res<FlowGrid>,
    heights: Res<TerrainHeightMap>,
) {
    let (entity, mut transform, mut session) = session.into_inner();
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Right) {
        commands.entity(entity).despawn_children().remove::<PlacementSession>();
        return;
    }
    if !buttons.pressed(MouseButton::Left) && !buttons.just_released(MouseButton::Left) {
        session.ready = true;
    }
    let (camera, camera_transform) = *camera;
    // The root camera's Transform includes this frame's camera motion; do not
    // use MouseState, which keeps its last terrain hit when the ray misses.
    let camera_global = GlobalTransform::from(*camera_transform);
    let hit = window.cursor_position()
        .filter(|_| !capture.pointer_over_ui)
        .and_then(|cursor| screen_to_world(camera, &camera_global, &heights, cursor));
    let Some(hit) = hit else {
        commands.entity(entity).insert(Visibility::Hidden);
        return;
    };
    commands.entity(entity).insert(Visibility::Visible);
    let origin = snapped_origin(Vec2::new(hit.x, hit.z), session.size, flow.cell_size);
    let center = footprint_center(origin, session.size, flow.cell_size);
    transform.translation = Vec3::new(center.x, hit.y, center.y);
    container.grid_pos = origin;
}

pub(super) fn confirm_placement(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    capture: Res<UiInputCapture>,
    session: Single<(Entity, &PlacementSession, &Transform, &Visibility)>,
    container: Single<&CellContainer>,
    cells: Query<&CellOverlay>,
    sprites: Res<SpriteCatalog>,
    mut flow: ResMut<FlowGrid>,
    mut spatial: ResMut<SpatialGrid>,
) {
    let (root, session, transform, visibility) = session.into_inner();
    if !session.ready
        || !buttons.just_pressed(MouseButton::Left)
        || capture.left_captured
        || *visibility != Visibility::Visible
    {
        return;
    }

    if cells.is_empty() || cells.iter().any(|cell| !cell.valid) {
        return;
    }

    let Some(visual) = sprites.sprites.get(&session.thing_type) else {
        return;
    };
    let origin = container.grid_pos;
    let center = footprint_center(origin, session.size, flow.cell_size);
    let id = match register_footprint(&mut flow, origin, session.size) {
        Ok(id) => id,
        Err(error) => {
            warn!("Building placement rejected: {error}");
            return;
        }
    };

    let building = commands
        .spawn((
            Building { id, size: session.size },
            session.thing_type,
            Position(center),
            Team::Player,
            Transform::from_translation(transform.translation),
        ))
        .id();
    spawn_billboard(&mut commands, visual, building, Some(Team::Player));
    spatial.update(&flow);
    commands.entity(root).despawn_children().remove::<PlacementSession>();
}

pub(super) fn update_building_transforms(
    mut buildings: Query<(&Position, &ThingType, &mut Transform), (With<Building>, Without<Camera3d>)>,
    camera: Single<&Transform, With<Camera3d>>,
    heights: Res<TerrainHeightMap>,
    sprites: Res<SpriteCatalog>,
) {
    for (position, thing_type, mut transform) in &mut buildings {
        if let Some(visual) = sprites.sprites.get(thing_type) {
            transform.translation = Vec3::new(position.x, heights.height_at(position.0), position.y)
                + camera.up().as_vec3() * (visual.size.y * 0.5);
        }
    }
}

pub(super) fn sync_spatial_obstacles(flow: Res<FlowGrid>, mut spatial: ResMut<SpatialGrid>) {
    spatial.update(&flow);
}

pub(super) fn remove_building_obstacle(
    event: On<Remove, Building>,
    buildings: Query<&Building>,
    mut flow: ResMut<FlowGrid>,
    mut spatial: ResMut<SpatialGrid>,
) {
    if let Ok(building) = buildings.get(event.entity) {
        match flow.map.remove_building(building.id) {
            Ok(()) => spatial.update(&flow),
            Err(error) => warn!("Could not remove building obstacle: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunk_flow_field::map::{Map, Side};

    fn flow() -> FlowGrid {
        FlowGrid { width: 8, height: 8, cell_size: 20.0, map: Map::new(&Obstacle::new(8, 8)) }
    }

    fn test_app() -> App {
        use crate::{building::BuildingPlugin, visual::{UnitVisual, UnitVisualKind}};
        let mut app = App::new();
        let flow = flow();
        let spatial = SpatialGrid::new(&flow);
        let mut meshes = Assets::<Mesh>::default();
        let mesh = meshes.add(Rectangle::new(60.0, 60.0));
        let mut team_materials = Assets::<TeamColorMaterial>::default();
        let material = team_materials.add(TeamColorMaterial::default());
        let mut sprites = SpriteCatalog::default();
        sprites.sprites.insert(ThingType::SupplyStorage, UnitVisual {
            kind: UnitVisualKind::Simple {
                mesh,
                material: [(Team::Player, material)].into_iter().collect(),
            },
            size: Vec2::splat(60.0),
            model_offset: Vec3::ZERO,
            anchor: None,
            roll_offset: 0.0,
        });
        app.add_plugins(MinimalPlugins)
            .insert_resource(flow)
            .insert_resource(spatial)
            .insert_resource(meshes)
            .insert_resource(team_materials)
            .insert_resource(Assets::<StandardMaterial>::default())
            .insert_resource(sprites)
            .insert_resource(ThingCatalog::new())
            .insert_resource(UiInputCapture::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(TerrainHeightMap {
                width: 9, height: 9, sample_spacing: 20.0, heights: vec![0.0; 81], mesh: None,
            })
            .add_plugins(BuildingPlugin);
        app.world_mut().spawn(Window::default());
        app.world_mut().spawn((Camera3d::default(), Transform::default()));
        app
    }

    fn entities_with<T: Component>(app: &mut App) -> Vec<Entity> {
        let world = app.world_mut();
        world.query_filtered::<Entity, With<T>>().iter(world).collect()
    }

    fn preview_cells(world: &World, root: Entity) -> &Children {
        let children = world.get::<Children>(root).unwrap();
        let container = children
            .iter()
            .find(|&child| world.get::<CellContainer>(child).is_some())
            .unwrap();
        world.get::<Children>(container).unwrap()
    }

    #[test]
    fn placement_can_occupy_last_row_and_column_of_grid() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        let world = app.world_mut();
        let mut cameras = world.query_filtered::<&mut Transform, With<Camera3d>>();
        *cameras.single_mut(world).unwrap() = Transform::from_xyz(150.0, 100.0, 150.0)
            .looking_at(Vec3::new(150.0, 0.0, 150.0), Vec3::NEG_Z);
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        assert!(entities_with::<PlacementSession>(&mut app).is_empty());
        let world = app.world_mut();
        let mut buildings = world.query::<(&Building, &Position)>();
        let (building, position) = buildings.single(world).unwrap();
        assert_eq!(building.size, UVec2::splat(2));
        assert!((position.0 - Vec2::splat(140.0)).length() < 0.001);
        let flow = world.resource::<FlowGrid>();
        let spatial = world.resource::<SpatialGrid>();
        for y in 6..8 {
            for x in 6..8 {
                assert!(matches!(flow.map.get_cell(x, y), Cell::Wall));
                assert!(spatial.obstacle_cells[y * 8 + x].is_none());
            }
        }
    }

    fn aim_at_map_center(app: &mut App) {
        use bevy::camera::RenderTargetInfo;
        let world = app.world_mut();
        let mut windows = world.query::<&mut Window>();
        let mut window = windows.single_mut(world).unwrap();
        window.resolution.set(800.0, 600.0);
        window.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
        let mut cameras = world.query_filtered::<(&mut Camera, &mut Transform), With<Camera3d>>();
        let (mut camera, mut transform) = cameras.single_mut(world).unwrap();
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: UVec2::new(800, 600), scale_factor: 1.0,
        });
        camera.computed.clip_from_view = Mat4::perspective_infinite_reverse_rh(
            std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 0.1,
        );
        *transform = Transform::from_xyz(70.0, 100.0, 70.0)
            .looking_at(Vec3::new(70.0, 0.0, 70.0), Vec3::NEG_Z);
    }

    #[test]
    fn model_offset_matches_preview_and_building_without_moving_footprint() {
        let offset = Vec3::new(3.0, 12.0, -5.0);
        let mut baseline = test_app();
        let mut shifted = test_app();
        shifted.world_mut().resource_mut::<SpriteCatalog>().sprites
            .get_mut(&ThingType::SupplyStorage).unwrap().model_offset = offset;
        for app in [&mut baseline, &mut shifted] {
            aim_at_map_center(app);
            app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
            app.update();
        }
        let ghost_transform = |app: &mut App| {
            let [ghost] = entities_with::<Ghost>(app)[..] else {
                panic!("expected one preview ghost");
            };
            *app.world().get::<Transform>(ghost).unwrap()
        };
        let original = ghost_transform(&mut baseline);
        let translated = ghost_transform(&mut shifted);
        assert!((translated.translation - original.translation - offset).length() < 0.001);
        assert_eq!(translated.rotation, original.rotation);
        // Cell overlays indicate the gameplay footprint, independently of the ghost.
        let cell_positions = |app: &mut App| {
            let root = entities_with::<PlacementSession>(app)[0];
            let world = app.world();
            preview_cells(world, root)
                .iter()
                .map(|cell| world.get::<Transform>(cell).unwrap().translation)
                .collect::<Vec<_>>()
        };
        assert_eq!(cell_positions(&mut baseline), cell_positions(&mut shifted));
        for app in [&mut baseline, &mut shifted] {
            app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
            app.update();
        }
        let installed = |app: &mut App| {
            let world = app.world_mut();
            let mut query = world.query::<(&Building, &Position, &Transform, &Children)>();
            let (building, position, parent, children) = query.single(world).unwrap();
            let billboard = children
                .iter()
                .find(|&child| world.get::<crate::world3d::Billboard>(child).is_some())
                .unwrap();
            let local = world.get::<Transform>(billboard).unwrap().translation;
            (building.size, position.0, parent.transform_point(local))
        };
        let original = installed(&mut baseline);
        let translated = installed(&mut shifted);
        assert_eq!(original.0, translated.0);
        assert_eq!(original.1, translated.1);
        assert!((translated.2 - original.2 - offset).length() < 0.001);
        for y in 0..8 {
            for x in 0..8 {
                let blocked = |app: &App| matches!(app.world().resource::<FlowGrid>().map.get_cell(x, y), Cell::Wall);
                assert_eq!(blocked(&baseline), blocked(&shifted));
            }
        }
        // Repeated camera-dependent updates must not rotate or accumulate the offset.
        for app in [&mut baseline, &mut shifted] {
            let world = app.world_mut();
            let mut cameras = world.query_filtered::<&mut Transform, With<Camera3d>>();
            cameras.single_mut(world).unwrap().rotation = Quat::from_rotation_x(-0.7);
            app.update();
            app.update();
        }
        let original = installed(&mut baseline);
        let translated = installed(&mut shifted);
        assert_eq!(original.1, translated.1);
        assert!((translated.2 - original.2 - offset).length() < 0.001);
    }

    #[test]
    fn click_installs_configured_footprint_and_syncs_collision() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();

        assert!(entities_with::<PlacementSession>(&mut app).is_empty());
        assert_eq!(entities_with::<BlockWorldInput>(&mut app).len(), 1);
        assert!(entities_with::<Ghost>(&mut app).is_empty());
        assert!(entities_with::<CellOverlay>(&mut app).is_empty());
        let world = app.world_mut();
        let mut buildings = world.query::<(&Building, &ThingType, &Position)>();
        let (building, kind, position) = buildings.single(world).unwrap();
        assert_eq!(building.size, UVec2::splat(2));
        assert_eq!(*kind, ThingType::SupplyStorage);
        assert!(position.0.distance(Vec2::splat(60.0)) < 0.01);
        let flow = world.resource::<FlowGrid>();
        let spatial = world.resource::<SpatialGrid>();
        for y in 2..4 {
            for x in 2..4 {
                assert!(matches!(flow.map.get_cell(x, y), Cell::Wall));
                assert!(spatial.obstacle_cells[y * 8 + x].is_none());
            }
        }
    }

    #[test]
    fn occupied_cell_is_red_and_click_does_not_install() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        // Center lies outside the footprint (40..80), but the radius overlaps it.
        let position = Vec2::new(81.0, 70.0);
        let entity = app.world_mut().spawn(Position(position)).id();
        app.world_mut().resource_mut::<SpatialGrid>().add_entity(entity, position, 3.0).unwrap();
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        let root = entities_with::<PlacementSession>(&mut app)[0];
        let world = app.world();
        let cell = preview_cells(world, root).iter().nth(3).unwrap();
        let overlay = world.get::<CellOverlay>(cell).unwrap();
        assert!(!overlay.valid);
        assert_eq!(world.get::<MeshMaterial3d<StandardMaterial>>(cell).unwrap().0, overlay.red);
        assert_eq!(app.world().resource::<FlowGrid>().map.get_generation(), 0);
    }

    #[test]
    fn circle_outside_footprint_corner_does_not_block_placement() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        // AABB overlaps the footprint corner, but the circle does not.
        let position = Vec2::splat(82.0);
        let entity = app.world_mut().spawn(Position(position)).id();
        app.world_mut().resource_mut::<SpatialGrid>().add_entity(entity, position, 2.5).unwrap();
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        assert!(entities_with::<PlacementSession>(&mut app).is_empty());
        let world = app.world_mut();
        assert_eq!(world.query::<&Building>().iter(world).count(), 1);
    }

    #[test]
    fn initiating_click_and_ui_click_cannot_install() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        assert_eq!(app.world().resource::<FlowGrid>().map.get_generation(), 0);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().reset_all();
        app.update();
        app.world_mut().resource_mut::<UiInputCapture>().pointer_over_ui = true;
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        let root = entities_with::<PlacementSession>(&mut app)[0];
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(), Visibility::Hidden);
        assert_eq!(app.world().resource::<FlowGrid>().map.get_generation(), 0);
    }

    #[test]
    fn beginning_and_cancelling_cleans_preview_and_owns_release() {
        let mut app = test_app();
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.world_mut().flush();
        let root = entities_with::<PlacementSession>(&mut app)[0];
        assert!(app.world().get::<BlockWorldInput>(root).is_some());
        assert_eq!(preview_cells(app.world(), root).len(), 4);

        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
        app.update();
        assert!(app.world().get::<PlacementSession>(root).is_none());
        assert!(app.world().get::<BlockWorldInput>(root).is_some());
        assert!(entities_with::<Ghost>(&mut app).is_empty());
        assert!(entities_with::<CellOverlay>(&mut app).is_empty());

        app.update();
        assert!(app.world().get::<BlockWorldInput>(root).is_some());
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Right);
        app.update();
        assert!(app.world().get::<BlockWorldInput>(root).is_some());
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().clear();
        app.update();
        assert!(app.world().get_entity(root).is_err());
    }

    #[test]
    fn escape_releases_world_input_without_a_mouse_event() {
        let mut app = test_app();
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.world_mut().flush();
        let root = entities_with::<PlacementSession>(&mut app)[0];
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        assert!(app.world().get::<PlacementSession>(root).is_none());
        assert!(app.world().get::<BlockWorldInput>(root).is_some());
        app.update();
        assert!(app.world().get_entity(root).is_err());
    }

    #[test]
    fn click_checks_cells_at_the_current_cursor_position() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        let root = entities_with::<PlacementSession>(&mut app)[0];
        assert!(preview_cells(app.world(), root)
            .iter()
            .all(|cell| app.world().get::<CellOverlay>(cell).unwrap().valid));

        // Move the cursor hit and click in the same frame. Only the new footprint overlaps.
        let world = app.world_mut();
        let mut cameras = world.query_filtered::<&mut Transform, With<Camera3d>>();
        *cameras.single_mut(world).unwrap() = Transform::from_xyz(110.0, 100.0, 70.0)
            .looking_at(Vec3::new(110.0, 0.0, 70.0), Vec3::NEG_Z);
        let position = Vec2::new(110.0, 70.0);
        let unit = world.spawn(Position(position)).id();
        world.resource_mut::<SpatialGrid>().add_entity(unit, position, 3.0).unwrap();
        world.resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();

        assert!(app.world().get::<PlacementSession>(root).is_some());
        assert!(preview_cells(app.world(), root)
            .iter()
            .any(|cell| !app.world().get::<CellOverlay>(cell).unwrap().valid));
        assert!(entities_with::<Building>(&mut app).is_empty());
    }

    #[test]
    fn despawning_a_building_restores_both_maps() {
        let mut app = test_app();
        let id = register_footprint(
            &mut app.world_mut().resource_mut::<FlowGrid>(),
            IVec2::ONE, UVec2::splat(3),
        ).unwrap();
        app.world_mut().run_system_cached(sync_spatial_obstacles).unwrap();
        assert!(app.world().resource::<SpatialGrid>().obstacle_cells[9].is_none());
        let entity = app.world_mut().spawn(Building { id, size: UVec2::splat(3) }).id();
        app.world_mut().despawn(entity);
        assert!(!matches!(app.world().resource::<FlowGrid>().map.get_cell(1, 1), Cell::Wall));
        assert!(app.world().resource::<SpatialGrid>().obstacle_cells[9].is_some());
    }

    #[test]
    fn snaps_odd_footprint_around_cursor_cell() {
        let size = UVec2::splat(3);
        let origin = snapped_origin(Vec2::new(49.0, 69.0), size, 20.0);
        assert_eq!(origin, IVec2::new(1, 2));
        assert_eq!(footprint_center(origin, size, 20.0), Vec2::new(50.0, 70.0));
    }

    fn validate_footprint(world: &mut World, origin: IVec2, size: IVec2) -> Vec<bool> {
        let container = world.spawn(CellContainer { size, grid_pos: origin }).id();
        let overlays: Vec<_> = (0..size.x * size.y)
            .map(|_| {
                world.spawn((
                    CellOverlay { green: Handle::default(), red: Handle::default(), valid: false },
                    ChildOf(container),
                )).id()
            })
            .collect();
        world.run_system_cached(cell_validate_system).unwrap();
        let valid = overlays
            .iter()
            .map(|&cell| world.get::<CellOverlay>(cell).unwrap().valid)
            .collect();
        world.despawn(container);
        valid
    }

    #[test]
    fn validates_each_cell_and_unit_radius_not_just_unit_center() {
        let mut app = test_app();
        let world = app.world_mut();
        assert_eq!(validate_footprint(world, IVec2::ONE, IVec2::splat(3)), vec![true; 9]);
        assert_eq!(validate_footprint(world, IVec2::new(-1, 0), IVec2::splat(2)),
            vec![false, true, false, true]);
        let unit = world.spawn_empty().id();
        world.resource_mut::<SpatialGrid>().add_entity(unit, Vec2::new(18.0, 30.0), 3.0).unwrap();
        let valid = validate_footprint(world, IVec2::ONE, IVec2::splat(3));
        assert!(!valid[0]);
        assert!(valid[1..].iter().all(|&v| v));
    }

    #[test]
    fn rejects_cells_beyond_rendered_terrain() {
        let mut app = test_app();
        let world = app.world_mut();
        // The last navigation column lies beyond the rendered terrain extent.
        let mut heights = world.resource_mut::<TerrainHeightMap>();
        heights.width = 8;
        heights.heights = vec![0.0; heights.width * heights.height];
        assert_eq!(validate_footprint(world, IVec2::new(7, 0), IVec2::ONE), vec![false]);
    }

    #[test]
    fn rejects_terrain_edges_and_existing_buildings() {
        let mut app = test_app();
        let world = app.world_mut();
        let mut terrain = Obstacle::new(8, 8);
        terrain.set_cell_blocked(1, 1);
        terrain.set_edge(3, 3, Side::Right);
        world.resource_mut::<FlowGrid>().map = Map::new(&terrain);
        let check = |world: &mut World, cell| validate_footprint(world, cell, IVec2::ONE)[0];
        assert!(!check(world, IVec2::ONE));
        assert!(!check(world, IVec2::new(3, 3)));
        let id = register_footprint(&mut world.resource_mut::<FlowGrid>(), IVec2::new(4, 4), UVec2::splat(2)).unwrap();
        assert!(!check(world, IVec2::new(4, 4)));
        assert!(register_footprint(&mut world.resource_mut::<FlowGrid>(), IVec2::new(4, 4), UVec2::ONE).is_err());
        world.resource_mut::<FlowGrid>().map.remove_building(id).unwrap();
        assert!(check(world, IVec2::new(4, 4)));
        assert!(register_footprint(&mut world.resource_mut::<FlowGrid>(), IVec2::new(7, 7), UVec2::splat(2)).is_err());
    }
}
