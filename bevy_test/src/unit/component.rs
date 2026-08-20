use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitStats {
    pub size: f32,
    pub max_speed: f32,
    pub acceleration: f32,
}

#[derive(Component, Debug, Default)]
pub struct Dead; // 유닛이 죽었는지 여부

#[derive(Component, Debug, Default)]
pub struct Selected; // 유닛이 선택되었는지 여부

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Team {
    Player,
    Enemy,
    Neutral,
    #[default]
    Empty,
}

impl Team {
    pub fn color(&self) -> Color {
        match self {
            Team::Player => Color::srgb(0.0, 0.5, 1.0),
            Team::Enemy => Color::srgb(1.0, 0.0, 0.0),
            Team::Neutral => Color::srgb(0.5, 0.5, 0.5),
            Team::Empty => Color::srgb(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Position(pub Vec2); // 유닛의 지상 위치 (x, y)
