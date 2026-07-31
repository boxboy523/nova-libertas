use crate::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*};

pub struct UIPlugin;

#[derive(Resource, Default)]
pub struct DragSelection {
    start: Vec2,
    current: Vec2,
    active: bool,
}

pub fn selection_system(
    mut command: Commands,
    mut drag_selection: ResMut<DragSelection>,
    state: Res<MouseState>,
    spatial_grid: Res<SpatialGrid>,
    query_select: Query<Entity, With<Selected>>,
) {
    if state.left_just_pressed {
        let Ok(units_at_cursor) = spatial_grid.query_entities(state.world_position, 1.0, false)
        else {
            return;
        };
        if let Some(unit) = units_at_cursor.first() {
            query_select.iter().for_each(|unit| {
                command.entity(unit).remove::<Selected>();
            });
            command.entity(unit.entity).insert(Selected);
        } else {
            drag_selection.start = state.world_position;
            drag_selection.current = state.world_position;
            drag_selection.active = true;
        }
    } else if state.left_pressed && drag_selection.active {
        drag_selection.current = state.world_position;
    } else if state.left_released && drag_selection.active {
        drag_selection.active = false;
        query_select.iter().for_each(|unit| {
            command.entity(unit).remove::<Selected>();
        });
        let min = drag_selection.start.min(drag_selection.current);
        let max = drag_selection.start.max(drag_selection.current);
        let selected_units = spatial_grid
            .query_entities_rect(min, max)
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|e| e.entity)
            .for_each(|unit| {
                command.entity(unit).insert(Selected);
            });
    }
}
