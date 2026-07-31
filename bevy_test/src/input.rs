use std::collections::HashSet;

use crate::prelude::*;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MouseState {
    pub window_position: Vec2,
    pub world_position: Vec2,
    pub left_just_pressed: bool,
    pub left_pressed: bool,
    pub left_released: bool,
    pub right_just_pressed: bool,
    pub right_pressed: bool,
    pub right_released: bool,
}

pub fn mouse_input(
    mut command: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut state: ResMut<MouseState>,
    spatial_grid: Res<SpatialGrid>,
    query_select: Query<Entity, (With<Selected>, With<UnitMovement>)>,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };
    state.window_position = cursor;
    state.world_position = world_position;

    if buttons.just_pressed(MouseButton::Left) {
        state.left_just_pressed = true;
        let Ok(units_at_cursor) = spatial_grid.query_entities(world_position, 1.0, false) else {
            return;
        };
        query_select.iter().for_each(|unit| {
            command.entity(unit).remove::<Selected>();
        });
        if let Some(unit) = units_at_cursor.first() {
            command.entity(unit.entity).insert(Selected);
        }
    } else {
        state.left_just_pressed = false;
    }
    state.left_pressed = buttons.pressed(MouseButton::Left);
    state.left_released = buttons.just_released(MouseButton::Left);

    if buttons.just_pressed(MouseButton::Right) {
        state.right_just_pressed = true;
        let selected_units: HashSet<Entity> = query_select.iter().collect();
        command.trigger(MoveOrderEvent {
            target_position: world_position,
            units: selected_units,
            auto_attack: false,
        });
    } else {
        state.right_just_pressed = false;
    }
    state.right_pressed = buttons.pressed(MouseButton::Right);
    state.right_released = buttons.just_released(MouseButton::Right);
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_input);
    }
}
