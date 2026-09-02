use std::collections::HashSet;

use bevy::prelude::*;
use bevy_hui::prelude::HtmlNode;

use crate::{
    input::MouseState,
    map::TerrainHeightMap,
    prelude::*,
    ui::components::{DragSelection, HpBarFill, HpBarRef, HpBarRoot},
};

pub fn spawn_selection_box(mut commands: Commands) {
    commands.spawn((
        DragSelection::default(),
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.65, 1.0, 0.3)),
        BorderColor::all(Color::srgba(0.2, 0.65, 1.0, 0.9)),
        ZIndex(100),
    ));
    println!("Spawned selection box");
}

pub fn spawn_test_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        HtmlNode(asset_server.load("ui/src/test_panel.html")),
        GlobalZIndex(100),
    ));
}

pub fn update_selection_box(selection_box: Single<(&mut Node, &DragSelection)>) {
    let (mut node, drag_selection) = selection_box.into_inner();
    if drag_selection.active {
        node.display = Display::Flex;
        let min = drag_selection.start.min(drag_selection.current);
        let max = drag_selection.start.max(drag_selection.current);
        node.left = Val::Px(min.x);
        node.top = Val::Px(min.y);
        node.width = Val::Px(max.x - min.x);
        node.height = Val::Px(max.y - min.y);
    } else {
        node.display = Display::None;
    }
}

pub fn selection_system(
    mut command: Commands,
    mut drag_selection: Single<&mut DragSelection>,
    state: Res<MouseState>,
    spatial_grid: Res<SpatialGrid>,
    height_map: Res<TerrainHeightMap>,
    query_select: Query<Entity, With<Selected>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    team_query: Query<&Team>,
) {
    if state.left_just_pressed {
        let Ok(units_at_cursor) = spatial_grid.query_entities(state.world_position, 1.0, false)
        else {
            warn!("Failed to query spatial grid for units at cursor");
            return;
        };
        if let Some(unit) = units_at_cursor.first() {
            if let Ok(team) = team_query.get(unit.entity) {
                if *team == Team::Player {
                    query_select.iter().for_each(|unit| {
                        command.entity(unit).remove::<Selected>();
                    });
                    command.entity(unit.entity).insert(Selected);
                } else if *team == Team::Enemy {
                    query_select.iter().for_each(|_| {
                        command.trigger(AttackOrderEvent {
                            target: unit.entity,
                            units: query_select.iter().collect::<HashSet<_>>(),
                        })
                    });
                }
            } else {
                warn!("Failed to get team for unit {:?}", unit.entity);
            }
        } else {
            drag_selection.start = state.window_position;
            drag_selection.current = state.window_position;
            drag_selection.active = true;
        }
    } else if state.left_pressed && drag_selection.active {
        drag_selection.current = state.window_position;
    } else if state.left_released && drag_selection.active {
        drag_selection.active = false;
        query_select.iter().for_each(|unit| {
            command.entity(unit).remove::<Selected>();
        });
        let min = drag_selection.start.min(drag_selection.current);
        let max = drag_selection.start.max(drag_selection.current);

        let screen_corners = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];

        let mut min_world = Vec2::splat(f32::INFINITY);
        let mut max_world = Vec2::splat(f32::NEG_INFINITY);

        for corner in screen_corners.iter() {
            if let Some(world_pos) = screen_to_ground(camera.0, camera.1, &height_map, *corner) {
                min_world = min_world.min(world_pos);
                max_world = max_world.max(world_pos);
            } else {
                warn!(
                    "Failed to convert screen position {:?} to world position",
                    corner
                );
                return;
            }
        }
        spatial_grid
            .query_entities_rect(min_world, max_world)
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .filter(|e| {
                let y = height_map.height_at(e.pos);
                let Ok(window_pos) = camera
                    .0
                    .world_to_viewport(camera.1, Vec3::new(e.pos.x, y, e.pos.y))
                else {
                    warn!(
                        "Failed to convert world position {:?} to screen position",
                        e.pos
                    );
                    return false;
                };
                window_pos.x >= min.x
                    && window_pos.x <= max.x
                    && window_pos.y >= min.y
                    && window_pos.y <= max.y
            })
            .map(|e| e.entity)
            .for_each(|unit| {
                if let Ok(team) = team_query.get(unit) {
                    if *team == Team::Player {
                        command.entity(unit).insert(Selected);
                    }
                }
            });
        println!(
            "Selected units in rectangle: {:?} to {:?}",
            min_world, max_world
        );
        drag_selection.start = Vec2::ZERO;
        drag_selection.current = Vec2::ZERO;
    }
}

pub fn move_selected_units(
    mut commands: Commands,
    state: Res<MouseState>,
    keys: Res<ButtonInput<KeyCode>>,
    query_select: Query<Entity, With<Selected>>,
) {
    let auto_attack = keys.just_pressed(KeyCode::KeyA);
    if state.right_just_pressed || auto_attack {
        let selected_units = query_select.iter().collect::<HashSet<_>>();
        if !selected_units.is_empty() {
            commands.trigger(MoveOrderEvent {
                target_position: state.world_position,
                units: selected_units,
                auto_attack: auto_attack,
            });
        }
    }
}

pub fn spawn_hp_bar(
    mut command: Commands,
    units: Query<(Entity, &ThingType), Added<UnitHp>>,
    catalog: Res<SpriteCatalog>,
) {
    units.iter().for_each(|(unit_entity, thing_type)| {
        let Some(visual) = catalog.sprites.get(thing_type) else {
            warn!("No visual found for thing type {:?}", thing_type);
            return;
        };

        let mut fill = Entity::PLACEHOLDER;

        let root = command
            .spawn((
                HpBarRoot {
                    owner: unit_entity,
                    visual_height: visual.size.y,
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(42.0),
                    height: px(6.0),
                    padding: UiRect::all(px(1.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                ZIndex(50),
            ))
            .with_children(|parent| {
                fill = parent
                    .spawn((
                        HpBarFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.9, 0.2)),
                    ))
                    .id();
            })
            .id();
        command.entity(unit_entity).insert(HpBarRef { root, fill });
    });
}

pub fn update_hp_bar_position_system(
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window>,
    units: Query<&GlobalTransform>,
    mut bars: Query<(&HpBarRoot, &mut Node)>,
) {
    let (camera, camera_transform) = *camera;

    bars.iter_mut().for_each(|(hp_bar_root, mut node)| {
        let Ok(unit_transform) = units.get(hp_bar_root.owner) else {
            node.display = Display::None;
            warn!(
                "Failed to get unit transform for entity {:?}",
                hp_bar_root.owner
            );
            return;
        };

        let world_pos =
            unit_transform.transform_point(Vec3::new(0.0, hp_bar_root.visual_height + 0.5, 0.0));

        let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            node.display = Display::None;
            warn!(
                "Failed to convert world position {:?} to screen position",
                world_pos
            );
            return;
        };

        if screen_pos.x < 0.0
            || screen_pos.x > window.width()
            || screen_pos.y < 0.0
            || screen_pos.y > window.height()
        {
            node.display = Display::None;
            return;
        }

        node.display = Display::Flex;
        node.left = px(screen_pos.x - 21.0);
        node.top = px(screen_pos.y - 3.0);
    });
}

pub fn update_hp_bar_fill_system(
    units: Query<(&UnitHp, &HpBarRef), Changed<UnitHp>>,
    mut fills: Query<&mut Node, With<HpBarFill>>,
) {
    units.iter().for_each(|(unit_hp, hp_bar)| {
        let Ok(mut fill_node) = fills.get_mut(hp_bar.fill) else {
            warn!(
                "Failed to get fill node for hp bar of entity {:?}",
                hp_bar.root
            );
            return;
        };
        let ratio = (unit_hp.current / unit_hp.max).clamp(0.0, 1.0);

        fill_node.width = percent(ratio * 100.0);
    });
}
