use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;

pub fn transform_update_system(
    query: Query<(&Transform, &TransformID)>,
    mut buffer: ResMut<TransformBuffer>,
) {
    let data = &mut buffer.data;
    query.iter().for_each(|(transform, id)| {
        let (sin, cos) = transform.rotation.sin_cos();
        let i = id.0 * 8; // TransformID를 인덱스로 사용

        data[i] = cos * transform.scale.x; // x.x
        data[i + 1] = sin * transform.scale.y; // y.x
        data[i + 2] = 0.0; // padding
        data[i + 3] = transform.position.x; // x.w (translation x)
        data[i + 4] = sin * transform.scale.x; // x.y
        data[i + 5] = -cos * transform.scale.y; // y.y
        data[i + 6] = 0.0; // padding
        data[i + 7] = transform.position.y; // y.w (translation y)
    });
}

pub fn despawn_units_system(
    mut commands: Commands,
    query: Query<(Entity, &TransformID), With<Dead>>,
    mut buffer: ResMut<TransformBuffer>,
) {
    for (entity, id) in query.iter() {
        commands.entity(entity).despawn();
        buffer.free(id.0); // 유닛이 죽으면 TransformID 인덱스 해제
    }
}
