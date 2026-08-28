use bevy::prelude::*;

use crate::map::TerrainHeightMap;

#[derive(Resource, Debug)]
pub struct MouseState {
    pub window_position: Vec2,
    pub world_position: Vec2,
    pub cursor_ray: Ray3d,
    pub left_just_pressed: bool,
    pub left_pressed: bool,
    pub left_released: bool,
    pub right_just_pressed: bool,
    pub right_pressed: bool,
    pub right_released: bool,
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState {
            window_position: Vec2::ZERO,
            world_position: Vec2::ZERO,
            cursor_ray: Ray3d::new(Vec3::ZERO, Dir3::X),
            left_just_pressed: false,
            left_pressed: false,
            left_released: false,
            right_just_pressed: false,
            right_pressed: false,
            right_released: false,
        }
    }
}

pub fn mouse_input(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    height_map: Res<TerrainHeightMap>,
    mut state: ResMut<MouseState>,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };

    let Some(hit) = height_map.raycast(&ray) else {
        warn!("Mouse ray did not hit the terrain height map");
        return;
    };
    state.world_position = Vec2::new(hit.x, hit.z);
    state.cursor_ray = ray;
    state.window_position = cursor;
    state.left_just_pressed = buttons.just_pressed(MouseButton::Left);
    state.left_pressed = buttons.pressed(MouseButton::Left);
    state.left_released = buttons.just_released(MouseButton::Left);
    state.right_just_pressed = buttons.just_pressed(MouseButton::Right);
    state.right_pressed = buttons.pressed(MouseButton::Right);
    state.right_released = buttons.just_released(MouseButton::Right);
}

pub fn screen_to_ground(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    height_map: &TerrainHeightMap,
    screen_position: Vec2,
) -> Option<Vec2> {
    let ray = camera
        .viewport_to_world(camera_transform, screen_position)
        .ok()?;

    let hit = height_map.raycast(&ray)?;
    Some(Vec2::new(hit.x, hit.z))
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_input);
    }
}
