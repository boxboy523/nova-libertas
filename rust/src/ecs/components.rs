use std::collections::HashSet;

use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;
use strum::EnumIter;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Transform {
    pub position: Vector2,
    pub rotation: f32,
    pub scale: Vector2,
    pub size: f32,           // 유닛의 크기 (반지름)
    pub buffer_index: usize, // TransformBuffer에서의 인덱스
    pub t_type: ThingType,   // 유닛의 종류 (ThingType)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UnitStats {
    pub max_speed: f32,
    pub acceleration: f32,
    pub max_hp: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UnitHp(pub f32); // 유닛의 현재 체력

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UnitMovement {
    pub speed: f32,             // 유닛의 현재 이동 속도
    pub preferred_speed: f32, // 유닛이 선호하는 이동 속도 (avoidance 등으로 인해 실제 이동 속도와 다를 수 있음)
    pub dir_vec: Vector2,     // 유닛이 현재 이동하는 방향 벡터
    pub preferred_dir: Vector2, // 유닛이 선호하는 이동 방향 (avoidance 등으로 인해 실제 이동 방향과 다를 수 있음)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Moving {
    pub order: Entity,       // 이 유닛이 따르는 MoveOrder 엔티티
    pub dist_target_sq: f32, // 목표 지점과의 직선거리 제곱
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Stopped {
    pub stop_position: Vector2, // 유닛이 멈춘 위치
    pub in_range: bool,         // stop_position과 범위 안에 있는지 여부
    pub pos_renew_delay: f32,   // stop_position 갱신 지연 시간 (초)
}

#[derive(Component, Debug, Default)]
pub struct Dead; // 유닛이 죽었는지 여부

#[derive(Component, Debug, Default)]
pub struct MoveOrder {
    pub target: Vector2,
    pub followers: HashSet<Entity>, // 이 명령을 따르는 유닛들
    pub following: HashSet<Entity>, // 이 명령을 따르는 유닛들 중 현재 따라가는 유닛들
    pub finished: HashSet<Entity>,  // 이 명령을 완료한 유닛들
}

#[derive(Component, Debug, Default)]
pub struct FlowField {
    pub goal: Vector2,
    pub field: Vec<Option<Vector2>>,
}

#[derive(Component, Debug)]
pub struct DelayedStopTrigger {
    pub timer: f32,
}

#[derive(Component, Debug, Default)]
pub struct Selected; // 유닛이 선택되었는지 여부

#[derive(Component, GodotConvert, Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
#[godot(via = i32)]
pub enum Team {
    Player,
    Enemy,
    #[default]
    Neutral,
}
