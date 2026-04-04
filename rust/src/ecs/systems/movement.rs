use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

pub fn movement_system(mut query: Query<(&mut Transform, &UnitStats)>, time: Res<Time>) {
    let delta = time.delta;
    for (mut pos, stats) in query.iter_mut() {
        // 임시 테스트용 우측 이동 로직
        pos.position.x += stats.speed * delta;
        pos.rotation += 0.01 * delta; // 약간씩 회전
        pos.scale = Vector2::new(1.0 + 0.5 * (pos.position.x / 100.0).sin(), 1.0);
    }
}
