use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::{
    map::TerrainHeightMap,
    ui::{
        BottomPanel, UiEntityIndex,
        events::{ToggleBottomPanelEvent, ToggleSidePanelEvent},
    },
};

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

#[derive(Resource, Debug, Default)]
pub struct UiInputCapture {
    pub mouse_wheel: bool,
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
        //warn!("Mouse ray did not hit the terrain height map");
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
        app.init_resource::<UiInputCapture>().add_systems(
            Update,
            (
                mouse_input,
                toggle_side_panel_input,
                scroll_panel_input,
            )
                .chain(),
        );
    }
}

pub fn scroll_panel_input(
    mut mouse_wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    ui_index: Res<UiEntityIndex>,
    mut scroll_nodes: Query<(
        &mut ScrollPosition,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut ui_input_capture: ResMut<UiInputCapture>,
) {
    ui_input_capture.mouse_wheel = false;

    let Some(cursor_position) = window.physical_cursor_position() else {
        mouse_wheel.clear();
        return;
    };
    let scroll_delta = mouse_wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => -event.y * 40.0,
            MouseScrollUnit::Pixel => -event.y,
        })
        .sum::<f32>();

    for viewport_id in [
        "build_icon_viewport",
        "unit_icon_viewport",
        "research_icon_viewport",
    ] {
        let Some(entity) = ui_index.entities.get(viewport_id) else {
            continue;
        };
        let Ok((mut scroll_position, computed_node, transform)) = scroll_nodes.get_mut(*entity)
        else {
            continue;
        };
        if !computed_node.contains_point(*transform, cursor_position) {
            continue;
        }

        ui_input_capture.mouse_wheel = true;
        let max_scroll = (computed_node.content_size().y - computed_node.size().y)
            * computed_node.inverse_scale_factor();
        scroll_position.y =
            (scroll_position.y + scroll_delta).clamp(0.0, max_scroll.max(0.0));
        break;
    }
}

pub fn toggle_side_panel_input(
    keys: Res<ButtonInput<KeyCode>>,
    ui_index: Res<UiEntityIndex>,
    transforms: Query<&UiTransform>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::ShiftLeft) {
        if let Some(entity) = ui_index.entities.get("side_panel") {
            if let Ok(transform) = transforms.get(*entity) {
                commands.trigger(ToggleSidePanelEvent(transform.translation.x));
            }
        }
    }

    let bottom_panel = if keys.just_pressed(KeyCode::KeyQ) {
        Some(BottomPanel::Build)
    } else if keys.just_pressed(KeyCode::KeyW) {
        Some(BottomPanel::Unit)
    } else if keys.just_pressed(KeyCode::KeyE) {
        Some(BottomPanel::Research)
    } else {
        None
    };

    if let Some(bottom_panel) = bottom_panel {
        commands.trigger(ToggleBottomPanelEvent(bottom_panel));
    }
}
