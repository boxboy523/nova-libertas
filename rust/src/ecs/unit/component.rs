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

#[derive(Component, Debug, Default)]
pub struct Dead; // 유닛이 죽었는지 여부

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
