use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UnitMovement {
    pub speed: f32,             // 유닛의 현재 이동 속도
    pub preferred_speed: f32, // 유닛이 선호하는 이동 속도 (avoidance 등으로 인해 실제 이동 속도와 다를 수 있음)
    pub dir_vec: Vector2,     // 유닛이 현재 이동하는 방향 벡터
    pub preferred_dir: Vector2, // 유닛이 선호하는 이동 방향 (avoidance 등으로 인해 실제 이동 방향과 다를 수 있음)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Moving {
    pub field: Entity,       // 이 유닛이 따르는 FlowField 엔티티
    pub dist_target_sq: f32, // 목표 지점과의 직선거리 제곱
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Stopped {
    pub stop_position: Vector2,     // 유닛이 멈춘 위치
    pub in_range: bool,             // stop_position과 범위 안에 있는지 여부
    pub pos_renew_delay: f32,       // stop_position 갱신 지연 시간 (초)
    pub last_field: Option<Entity>, // 마지막으로 따랐던 FlowField 엔티티
}

#[derive(Component, Debug)]
pub struct DelayedStopTrigger {
    pub timer: f32,
}

#[derive(Component, Debug, Default)]
pub struct FlowField {
    pub goal: Vector2,
    pub field: Vec<Option<Vector2>>,
}

#[derive(Component, Debug)]
pub struct FieldFollowTarget(pub Entity); // FlowField가 따라야 하는 대상 유닛 엔티티
