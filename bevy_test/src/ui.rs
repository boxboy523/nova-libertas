use std::collections::HashSet;

use crate::prelude::*;
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_selection_box).add_systems(
            Update,
            (selection_system, update_selection_box, move_selected_units)
                .chain()
                .after(mouse_input),
        );
    }
}

#[derive(Component, Default)]
pub struct DragSelection {
    start: Vec2,
    current: Vec2,
    active: bool,
}

fn spawn_selection_box(mut commands: Commands) {
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

fn update_selection_box(selection_box: Single<(&mut Node, &DragSelection)>) {
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

fn selection_system(
    mut command: Commands,
    mut drag_selection: Single<&mut DragSelection>,
    state: Res<MouseState>,
    spatial_grid: Res<SpatialGrid>,
    query_select: Query<Entity, With<Selected>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if state.left_just_pressed {
        let Ok(units_at_cursor) = spatial_grid.query_entities(state.world_position, 1.0, false)
        else {
            warn!("Failed to query spatial grid for units at cursor");
            return;
        };
        if let Some(unit) = units_at_cursor.first() {
            query_select.iter().for_each(|unit| {
                command.entity(unit).remove::<Selected>();
            });
            command.entity(unit.entity).insert(Selected);
            println!("Selected unit at cursor: {:?}", unit.entity);
        } else {
            drag_selection.start = state.window_position;
            drag_selection.current = state.window_position;
            drag_selection.active = true;
            println!("Started drag selection at: {:?}", drag_selection.start);
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

        let world_start = camera
            .0
            .viewport_to_world_2d(camera.1, min)
            .unwrap_or(Vec2::ZERO);
        let world_end = camera
            .0
            .viewport_to_world_2d(camera.1, max)
            .unwrap_or(Vec2::ZERO);
        let min_world = world_start.min(world_end);
        let max_world = world_start.max(world_end);
        spatial_grid
            .query_entities_rect(min_world, max_world)
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|e| e.entity)
            .for_each(|unit| {
                command.entity(unit).insert(Selected);
            });
    }
}

fn move_selected_units(
    mut commands: Commands,
    state: Res<MouseState>,
    query_select: Query<Entity, With<Selected>>,
) {
    if state.right_just_pressed {
        let selected_units = query_select.iter().collect::<HashSet<_>>();
        if !selected_units.is_empty() {
            commands.trigger(MoveOrderEvent {
                target_position: state.world_position,
                units: selected_units,
                auto_attack: false,
            });
        }
    }
}
