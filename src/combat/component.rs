use crate::{thing::ThingId, unit::component::Team};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitBattleStats {
    pub range: f32,
    pub damage: f32,
    pub cooldown: f32,
    pub delivery: AttackDelivery,
    pub impact: AttackImpact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttackDelivery {
    Instant,
    Projectile { speed: f32, id: ThingId },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttackImpact {
    Single,
    Area { radius: f32 },
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Durability {
    pub current_hp: f32,
    pub max_hp: f32,
    pub armor: f32,
}

impl Durability {
    pub fn new(stats: &DurabilityStats, hp: Option<f32>) -> Self {
        Self {
            current_hp: hp.unwrap_or(stats.max_hp),
            max_hp: stats.max_hp,
            armor: stats.armor,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DurabilityStats {
    pub max_hp: f32,
    pub armor: f32,
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
    pub speed: f32,           // 투사체 속도
    pub impact: AttackImpact, // 투사체의 공격 효과
    pub team: Team,           // 발사한 유닛의 팀
}
