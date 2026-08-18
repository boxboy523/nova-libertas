use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitBattleStats {
    pub range: f32,
    pub damage: f32,
    pub cooldown: f32,
    pub attack_type: AttackType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttackType {
    Instant,
    Projectile { speed: f32 },
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

#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub sender: Entity, // 발사한 유닛 엔티티
    pub target: Entity, // 공격 대상 유닛 엔티티
    pub damage: f32,
    pub speed: f32, // 투사체 속도
}
