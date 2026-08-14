use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitBattleStats {
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UnitHp {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Attack {
    pub target: Entity,  // 공격 대상 유닛 엔티티
    pub cooldown: f32,   // 공격 쿨다운 시간 (초)
    pub attacking: bool, // 현재 공격 중인지 여부
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AutoAttack;
