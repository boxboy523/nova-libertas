use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

pub fn transform_update_system(
    mut query: Query<(&Transform, &TransformID), Changed<Transform>>,
    mut buffer: ResMut<TransformBuffer>,
) {
    query.iter_mut().for_each(|(transform, id)| {
        let (sin, cos) = transform.rotation.sin_cos();
        let i = id.0 * 8;
        let chunk_index = id.0 / CHUNK_SIZE;
        buffer.chunks[chunk_index].modified = true;

        buffer.data[i] = cos * transform.scale.x; // x.x
        buffer.data[i + 1] = sin * transform.scale.y; // y.x
        buffer.data[i + 2] = 0.0; // padding
        buffer.data[i + 3] = transform.position.x; // x.w (translation x)
        buffer.data[i + 4] = sin * transform.scale.x; // x.y
        buffer.data[i + 5] = -cos * transform.scale.y; // y.y
        buffer.data[i + 6] = 0.0; // padding
        buffer.data[i + 7] = transform.position.y; // y.w (translation y)
    });
}

pub fn despawn_units_system(
    mut commands: Commands,
    query: Query<(Entity, &TransformID), With<Dead>>,
    mut buffer: ResMut<TransformBuffer>,
) {
    for (entity, id) in query.iter() {
        commands.entity(entity).despawn();
        buffer.free(id.0);
    }
}
