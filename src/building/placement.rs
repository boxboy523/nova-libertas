use bevy::{camera::visibility::NoFrustumCulling, light::NotShadowCaster, mesh::VertexAttributeValues, prelude::*};
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
    unit::{component::{Dead, Position, Team, UnitStats}, spatial_grid::SpatialGrid},
    visual::{SpriteCatalog, team_color::TeamColorMaterial},
    world3d::spawn_billboard,
};

#[derive(Event)]
pub struct BeginBuildingPlacement(pub ThingType);

#[derive(Resource, Default)]
pub struct BuildingPlacementState {
    active: Option<PlacementSession>,
    // Keep ownership of the final click/cancel gesture through its release frame.
    block_world_input: bool,
}

impl BuildingPlacementState {
    pub fn blocks_world_input(&self) -> bool {
        self.block_world_input || self.active.is_some()
    }
}

struct PlacementSession {
    thing_type: ThingType,
    size: UVec2,
    ghost: Entity,
    cells: Vec<Entity>,
    cell_meshes: Vec<Handle<Mesh>>,
    // Ghost, valid-cell and invalid-cell materials; never mutate catalog materials.
    materials: [Handle<StandardMaterial>; 3],
    armed: bool,
}

impl PlacementSession {
    fn cleanup(
        self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        commands.entity(self.ghost).despawn();
        for entity in self.cells {
            commands.entity(entity).despawn();
        }
        for mesh in self.cell_meshes {
            meshes.remove(mesh.id());
        }
        for material in self.materials {
            materials.remove(material.id());
        }
    }
}

pub(super) fn begin_placement(
    event: On<BeginBuildingPlacement>,
    mut commands: Commands,
    mut state: ResMut<BuildingPlacementState>,
    catalog: Res<ThingCatalog>,
    sprites: Res<SpriteCatalog>,
    team_materials: Res<Assets<TeamColorMaterial>>,
    flow_grid: Res<FlowGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(stats) = catalog.get_info(event.0).and_then(|info| info.building_stats) else {
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
    let green = materials.add(cell_material(Color::srgba(0.1, 1.0, 0.2, 0.4)));
    let red = materials.add(cell_material(Color::srgba(1.0, 0.1, 0.1, 0.4)));
    if let Some(old) = state.active.take() {
        old.cleanup(&mut commands, &mut meshes, &mut materials);
    }
    let ghost = commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(ghost_material.clone()),
        Transform::default(),
        Visibility::Hidden,
        Pickable::IGNORE,
        NotShadowCaster,
    )).id();
    let mut cells = Vec::new();
    let mut cell_meshes = Vec::new();
    for _ in 0..stats.size.x * stats.size.y {
        let mesh = meshes.add(Plane3d::default().mesh().size(
            flow_grid.cell_size * 0.96,
            flow_grid.cell_size * 0.96,
        ));
        cells.push(commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(red.clone()),
            Transform::default(),
            Visibility::Hidden,
            Pickable::IGNORE,
            NotShadowCaster,
            // Vertices follow terrain heights after the mesh's initial AABB is computed.
            NoFrustumCulling,
        )).id());
        cell_meshes.push(mesh);
    }
    state.active = Some(PlacementSession {
        thing_type: event.0,
        size: stats.size,
        ghost,
        cells,
        cell_meshes,
        materials: [ghost_material, green, red],
        armed: false,
    });
    state.block_world_input = true;
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

/// The cell under the cursor is the central cell for an odd-sized footprint.
fn snapped_origin(position: Vec2, size: UVec2, cell_size: f32) -> IVec2 {
    (position / cell_size).floor().as_ivec2() - (size / 2).as_ivec2()
}

fn footprint_center(origin: IVec2, size: UVec2, cell_size: f32) -> Vec2 {
    (origin.as_vec2() + size.as_vec2() * 0.5) * cell_size
}

/// Check the whole footprint against the current map and current unit positions,
/// not the potentially older fixed-tick spatial index.
fn valid_cells(
    flow: &FlowGrid,
    origin: IVec2,
    size: UVec2,
    terrain_size: Vec2,
    units: &[(Vec2, f32)],
) -> Vec<bool> {
    let mut valid = Vec::with_capacity((size.x * size.y) as usize);
    for dy in 0..size.y {
        for dx in 0..size.x {
            let cell = origin + IVec2::new(dx as i32, dy as i32);
            let min = cell.as_vec2() * flow.cell_size;
            let max = min + Vec2::splat(flow.cell_size);
            let in_bounds = cell.x >= 0 && cell.y >= 0
                && (cell.x as usize) < flow.map.width
                && (cell.y as usize) < flow.map.height
                && max.x <= terrain_size.x && max.y <= terrain_size.y;
            let free = in_bounds
                && !matches!(flow.map.get_cell(cell.x as usize, cell.y as usize), Cell::Wall)
                && flow.map.can_pass(cell.x as usize, cell.y as usize,
                    EDGE_TOP | EDGE_RIGHT | EDGE_BOTTOM | EDGE_LEFT)
                && !units.iter().any(|(position, radius)| {
                    position.distance_squared(position.clamp(min, max)) <= radius * radius
                });
            valid.push(free);
        }
    }
    valid
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

pub(super) fn update_placement(
    mut commands: Commands,
    mut state: ResMut<BuildingPlacementState>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<UiInputCapture>,
    window: Single<&Window>,
    camera: Single<(&Camera, &Transform), With<Camera3d>>,
    mut flow: ResMut<FlowGrid>,
    mut spatial: ResMut<SpatialGrid>,
    heights: Res<TerrainHeightMap>,
    units: Query<(&Position, &UnitStats), Without<Dead>>,
    sprites: Res<SpriteCatalog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut session) = state.active.take() else {
        if !buttons.pressed(MouseButton::Left) && !buttons.pressed(MouseButton::Right)
            && !buttons.just_released(MouseButton::Left) && !buttons.just_released(MouseButton::Right)
        {
            state.block_world_input = false;
        }
        return;
    };
    state.block_world_input = true;
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Right) {
        session.cleanup(&mut commands, &mut meshes, &mut materials);
        return;
    }
    if !buttons.pressed(MouseButton::Left) && !buttons.just_released(MouseButton::Left) {
        session.armed = true;
    }
    let (camera, camera_transform) = *camera;
    // The root camera's Transform includes this frame's camera motion; do not
    // use MouseState, which keeps its last terrain hit when the ray misses.
    let camera_global = GlobalTransform::from(*camera_transform);
    let hit = window.cursor_position()
        .filter(|_| !capture.pointer_over_ui)
        .and_then(|cursor| camera.viewport_to_world(&camera_global, cursor).ok())
        .and_then(|ray| heights.raycast(&ray));
    let Some(hit) = hit else {
        for entity in std::iter::once(session.ghost).chain(session.cells.iter().copied()) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        state.active = Some(session);
        return;
    };
    let origin = snapped_origin(Vec2::new(hit.x, hit.z), session.size, flow.cell_size);
    let center = footprint_center(origin, session.size, flow.cell_size);
    let unit_circles = units.iter().map(|(position, stats)| (position.0, stats.size)).collect::<Vec<_>>();
    let terrain_size = Vec2::new(
        heights.width.saturating_sub(1) as f32 * heights.sample_spacing,
        heights.height.saturating_sub(1) as f32 * heights.sample_spacing,
    );
    let valid = valid_cells(&flow, origin, session.size, terrain_size, &unit_circles);
    let visual = &sprites.sprites[&session.thing_type];
    // Simple catalog quads are centered. Lift along camera-up so the bottom
    // of the image stays anchored to the footprint's ground center.
    let translation = Vec3::new(center.x, heights.height_at(center), center.y)
        + camera_transform.up().as_vec3() * (visual.size.y * 0.5)
        + visual.world_offset;
    commands.entity(session.ghost).insert((
        Visibility::Visible,
        Transform::from_translation(translation).with_rotation(camera_transform.rotation),
    ));
    for (index, &entity) in session.cells.iter().enumerate() {
        let cell = origin + IVec2::new(
            index as i32 % session.size.x as i32,
            index as i32 / session.size.x as i32,
        );
        let center = (cell.as_vec2() + Vec2::splat(0.5)) * flow.cell_size;
        if let Some(mut mesh) = meshes.get_mut(&session.cell_meshes[index]) {
            if let Some(VertexAttributeValues::Float32x3(vertices)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                for vertex in vertices {
                    vertex[1] = heights.height_at(center + Vec2::new(vertex[0], vertex[2])) + 0.6;
                }
            }
        }
        commands.entity(entity).insert((
            Visibility::Visible,
            Transform::from_xyz(center.x, 0.0, center.y),
            MeshMaterial3d(session.materials[if valid[index] { 1 } else { 2 }].clone()),
        ));
    }
    if session.armed && buttons.just_pressed(MouseButton::Left)
        && !capture.left_captured && valid.iter().all(|&cell| cell)
    {
        // This check and registration use the same current resources in one system.
        match register_footprint(&mut flow, origin, session.size) {
            Ok(id) => {
                let entity = commands.spawn((
                    Building { id, size: session.size },
                    session.thing_type,
                    Position(center),
                    Team::Player,
                    Transform::from_translation(translation),
                )).id();
                spawn_billboard(&mut commands, visual, entity, Some(Team::Player));
                spatial.update(&flow);
                session.cleanup(&mut commands, &mut meshes, &mut materials);
                return;
            }
            Err(error) => warn!("Building placement rejected: {error}"),
        }
    }
    state.active = Some(session);
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
                + camera.up().as_vec3() * (visual.size.y * 0.5)
                + visual.world_offset;
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
            world_offset: Vec3::ZERO,
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
                width: 8, height: 8, sample_spacing: 20.0, heights: vec![0.0; 64], mesh: None,
            })
            .add_plugins(BuildingPlugin);
        app.world_mut().spawn(Window::default());
        app.world_mut().spawn((Camera3d::default(), Transform::default()));
        app
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
    fn world_offset_matches_preview_and_building_without_moving_footprint() {
        let offset = Vec3::new(3.0, 12.0, -5.0);
        let mut baseline = test_app();
        let mut shifted = test_app();
        shifted.world_mut().resource_mut::<SpriteCatalog>().sprites
            .get_mut(&ThingType::SupplyStorage).unwrap().world_offset = offset;
        for app in [&mut baseline, &mut shifted] {
            aim_at_map_center(app);
            app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
            app.update();
        }
        let ghost_transform = |app: &App| {
            let session = app.world().resource::<BuildingPlacementState>().active.as_ref().unwrap();
            *app.world().get::<Transform>(session.ghost).unwrap()
        };
        let original = ghost_transform(&baseline);
        let translated = ghost_transform(&shifted);
        assert!((translated.translation - original.translation - offset).length() < 0.001);
        assert_eq!(translated.rotation, original.rotation);
        // Cell overlays are gameplay footprint indicators, not part of the sprite.
        let cell_positions = |app: &App| {
            app.world().resource::<BuildingPlacementState>().active.as_ref().unwrap().cells
                .iter().map(|&cell| app.world().get::<Transform>(cell).unwrap().translation)
                .collect::<Vec<_>>()
        };
        assert_eq!(cell_positions(&baseline), cell_positions(&shifted));
        for app in [&mut baseline, &mut shifted] {
            app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
            app.update();
        }
        let installed = |app: &mut App| {
            let world = app.world_mut();
            let mut query = world.query::<(&Building, &Position, &Transform)>();
            let (building, position, transform) = query.single(world).unwrap();
            (building.size, position.0, transform.translation)
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

        assert!(app.world().resource::<BuildingPlacementState>().active.is_none());
        assert!(app.world().resource::<BuildingPlacementState>().blocks_world_input());
        let world = app.world_mut();
        let mut buildings = world.query::<(&Building, &ThingType, &Position)>();
        let (building, kind, position) = buildings.single(world).unwrap();
        assert_eq!(building.size, UVec2::splat(3));
        assert_eq!(*kind, ThingType::SupplyStorage);
        assert!(position.0.distance(Vec2::splat(70.0)) < 0.01);
        let flow = world.resource::<FlowGrid>();
        let spatial = world.resource::<SpatialGrid>();
        for y in 2..5 {
            for x in 2..5 {
                assert!(matches!(flow.map.get_cell(x, y), Cell::Wall));
                assert!(spatial.obstacle_cells[y * 8 + x].is_none());
            }
        }
        assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), 0);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), 1);
    }

    #[test]
    fn occupied_cell_is_red_and_click_does_not_install() {
        let mut app = test_app();
        aim_at_map_center(&mut app);
        app.world_mut().spawn((Position(Vec2::splat(70.0)), UnitStats {
            size: 5.0, max_speed: 10.0, acceleration: 10.0,
        }));
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.update();
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        app.update();
        let state = app.world().resource::<BuildingPlacementState>();
        let session = state.active.as_ref().unwrap();
        let red = &session.materials[2];
        assert_eq!(&app.world().get::<MeshMaterial3d<StandardMaterial>>(session.cells[4]).unwrap().0, red);
        assert_eq!(app.world().resource::<FlowGrid>().map.get_generation(), 0);
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
        let state = app.world().resource::<BuildingPlacementState>();
        let session = state.active.as_ref().unwrap();
        assert_eq!(*app.world().get::<Visibility>(session.ghost).unwrap(), Visibility::Hidden);
        assert_eq!(app.world().resource::<FlowGrid>().map.get_generation(), 0);
    }

    #[test]
    fn beginning_and_cancelling_cleans_preview_and_owns_release() {
        let mut app = test_app();
        app.world_mut().trigger(BeginBuildingPlacement(ThingType::SupplyStorage));
        app.world_mut().flush();
        assert!(app.world().resource::<BuildingPlacementState>().blocks_world_input());
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 3);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 10);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
        app.update();
        assert!(app.world().resource::<BuildingPlacementState>().active.is_none());
        assert!(app.world().resource::<BuildingPlacementState>().blocks_world_input());
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 0);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().reset_all();
        app.update();
        assert!(!app.world().resource::<BuildingPlacementState>().blocks_world_input());
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

    #[test]
    fn validates_each_cell_and_unit_radius_not_just_unit_center() {
        let flow = flow();
        let valid = valid_cells(&flow, IVec2::ONE, UVec2::splat(3), Vec2::splat(140.0), &[]);
        assert_eq!(valid, vec![true; 9]);
        let valid = valid_cells(&flow, IVec2::ONE, UVec2::splat(3), Vec2::splat(140.0), &[(Vec2::new(18.0, 30.0), 3.0)]);
        assert!(!valid[0]);
        assert!(valid[1..].iter().all(|&v| v));
        assert!(valid_cells(&flow, IVec2::new(-1, 0), UVec2::ONE, Vec2::splat(140.0), &[]).iter().all(|&v| !v));
        // Last navigation cell lies beyond the rendered terrain extent.
        assert_eq!(valid_cells(&flow, IVec2::new(7, 0), UVec2::ONE, Vec2::splat(140.0), &[]), vec![false]);
    }

    #[test]
    fn rejects_terrain_edges_and_existing_buildings() {
        let mut terrain = Obstacle::new(8, 8);
        terrain.set_cell_blocked(1, 1);
        terrain.set_edge(3, 3, Side::Right);
        let mut flow = FlowGrid { map: Map::new(&terrain), ..flow() };
        let check = |flow: &FlowGrid, cell| valid_cells(flow, cell, UVec2::ONE, Vec2::splat(140.0), &[])[0];
        assert!(!check(&flow, IVec2::ONE));
        assert!(!check(&flow, IVec2::new(3, 3)));
        let id = register_footprint(&mut flow, IVec2::new(4, 4), UVec2::splat(2)).unwrap();
        assert!(!check(&flow, IVec2::new(4, 4)));
        assert!(register_footprint(&mut flow, IVec2::new(4, 4), UVec2::ONE).is_err());
        flow.map.remove_building(id).unwrap();
        assert!(check(&flow, IVec2::new(4, 4)));
        assert!(register_footprint(&mut flow, IVec2::new(7, 7), UVec2::splat(3)).is_err());
    }
}
