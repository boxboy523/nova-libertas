use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

pub fn transform_update_system(
    query: Query<&Transform, Changed<Transform>>,
    mut buffer: ResMut<TransformBuffer>,
) {
    query.iter().for_each(|transform| {
        buffer.update(*transform);
    });
}

pub fn despawn_units_system(
    mut commands: Commands,
    dead_query: Query<Entity, With<Dead>>,
    mut query: Query<&mut Transform>,
    mut buffer: ResMut<TransformBuffer>,
) {
    for entity in dead_query.iter() {
        let Ok(transform) = query.get(entity) else {
            godot_print!(
                "despawn_units_system: Entity {:?} has no Transform component",
                entity
            );
            continue;
        };
        commands.entity(entity).despawn();
        if let Some(info) = buffer.delete(*transform) {
            // Update the swapped entity's Transform component with the new buffer index
            if let Ok(mut transform) = query.get_mut(info.swapped_entity) {
                transform.buffer_index = info.swapped_index;
            }
        }
    }
}
