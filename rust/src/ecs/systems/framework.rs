use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

pub fn transform_update_system(
    query: Query<(&Transform, Option<&UnitMovement>, Option<&Team>), Changed<Transform>>,
    query_hp: Query<(&Transform, &UnitHp, &UnitStats), Or<(Changed<UnitHp>, Changed<Transform>)>>,
    mut buffer: ResMut<TransformBuffer>,
) {
    query.iter().for_each(|(transform, movement, team)| {
        buffer.update(*transform, movement.map(|m| m.preferred_dir), team.copied());
    });
    query_hp.iter().for_each(|(transform, hp, stats)| {
        buffer.update_hp(*transform, hp.0 / stats.max_hp);
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
