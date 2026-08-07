use crate::prelude::*;
use bevy::prelude::*;

pub fn move_or_attack_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Position,
        &mut Attack,
        &UnitBattleStats,
        Option<&AutoAttack>,
    )>,
    query_position: Query<&Position>,
) {
    query
        .iter_mut()
        .for_each(|(entity, position, mut attack, battle_stats, opt_auto)| {
            if let Ok(target_position) = query_position.get(attack.target) {
                let dist_sq = position.distance_squared(**target_position);
                if dist_sq < battle_stats.attack_range * battle_stats.attack_range {
                    attack.attacking = true;
                } else {
                    attack.attacking = false;
                    if opt_auto.is_some() {
                        commands.entity(entity).remove::<Attack>();
                    }
                }
            }
        });
}

pub fn auto_attack_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Position, &UnitBattleStats), (With<AutoAttack>, Without<Attack>)>,
    query_position: Query<&Position>,
    query_team: Query<&Team>,
    spatial_grid: Res<SpatialGrid>,
) {
    query
        .iter_mut()
        .for_each(|(entity, position, battle_stats)| {
            let team = if let Ok(team) = query_team.get(entity) {
                team
            } else {
                return; // 팀 정보를 가져올 수 없으면 건너뜀
            };
            let mut nearby_entities = if let Ok(entity_info_vec) = spatial_grid.query_entities(
                **position,
                battle_stats.attack_range,
                true,
            ) {
                entity_info_vec
                    .into_iter()
                    .map(|e| e.entity)
                    .filter(|&e| e != entity && query_team.get(e).map_or(false, |t| t != team))
                    .collect::<Vec<_>>()
            } else {
                warn!("SpatialGrid query_entities failed for entity {:?}", entity);
                vec![]
            };
            nearby_entities.sort_by(|&a, &b| {
                let dist_a = query_position.get(a).map_or(f32::MAX, |target_position| {
                    position.distance_squared(**target_position)
                });
                let dist_b = query_position.get(b).map_or(f32::MAX, |target_position| {
                    position.distance_squared(**target_position)
                });
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(&closest_enemy) = nearby_entities.first() {
                commands.entity(entity).insert(Attack {
                    target: closest_enemy,
                    attacking: false,
                    cooldown: 0.0,
                });
            } else {
                commands.entity(entity).remove::<Attack>();
            }
        });
}

pub fn attack_system(
    mut commands: Commands,
    mut attackers: Query<(Entity, &mut Attack, &mut UnitMovement, &UnitBattleStats)>,
    query_position: Query<&Position>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    attackers
        .iter_mut()
        .for_each(|(entity, mut attack, mut movement, battle_stats)| {
            if attack.attacking == false {
                return;
            }
            let position = match query_position.get(entity) {
                Ok(position) => position,
                Err(_) => return, // Position 컴포넌트가 없으면 스킵
            };
            let dir = if let Ok(target_position) = query_position.get(attack.target) {
                (**position - **target_position).normalize_or_zero()
            } else {
                Vec2::ZERO
            };
            movement.dir_vec = dir;
            // 공격 쿨다운 감소
            attack.cooldown -= delta_time;
            if attack.cooldown <= 0.0 {
                commands.trigger(DamageEvent {
                    sender: entity,
                    receiver: attack.target,
                    damage: battle_stats.attack_damage,
                });
                // 공격 후 쿨다운 초기화
                attack.cooldown = battle_stats.attack_cooldown;
            }
        });
}
