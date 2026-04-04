use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub position: Vector2,
    pub rotation: f32,
    pub scale: Vector2,
}

#[derive(Component)]
pub struct TransformID(pub usize); // 유닛마다 고유한 Transform 버퍼 인덱스 (필요 시)

#[derive(Component, Clone, Copy)]
pub struct UnitMovement {
    pub speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub moving: bool,
}

#[derive(Component)]
pub struct Dead; // 유닛이 죽었는지 여부

#[derive(Component)]
pub struct MoveOrder {
    pub target: Vector2,
    pub followers: Vec<Entity>, // 이 명령을 따르는 유닛들
}

#[derive(Component)]
pub struct FlowField {
    pub field: Vec<Vector2>,
}
