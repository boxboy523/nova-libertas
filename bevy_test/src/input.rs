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
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut state: ResMut<MouseState>,
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

    state.left_just_pressed = buttons.just_pressed(MouseButton::Left);
    state.left_pressed = buttons.pressed(MouseButton::Left);
    state.left_released = buttons.just_released(MouseButton::Left);
    state.right_just_pressed = buttons.just_pressed(MouseButton::Right);
    state.right_pressed = buttons.pressed(MouseButton::Right);
    state.right_released = buttons.just_released(MouseButton::Right);
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_input);
    }
}
