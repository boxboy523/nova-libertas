use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitStats {
    pub size: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub max_hp: f32,
}

#[derive(Component, Debug, Default)]
pub struct Dead; // 유닛이 죽었는지 여부

#[derive(Component, Debug, Default)]
pub struct Selected; // 유닛이 선택되었는지 여부

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Team {
    Player,
    Enemy,
    #[default]
    Neutral,
}
