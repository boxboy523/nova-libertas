use crate::prelude::*;
use bevy::prelude::*;

pub fn move_or_attack_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Transform,
        &mut Attack,
        &UnitBattleStats,
        Option<&AutoAttack>,
    )>,
    query_transform: Query<&Transform>,
) {
    query
        .iter_mut()
        .for_each(|(entity, transform, mut attack, battle_stats, opt_auto)| {
            if let Ok(target_transform) = query_transform.get(attack.target) {
                let dist_sq = transform
                    .translation
                    .xy()
                    .distance_squared(target_transform.translation.xy());
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
    mut query: Query<(Entity, &Transform, &UnitBattleStats), (With<AutoAttack>, Without<Attack>)>,
    query_transform: Query<&Transform>,
    query_team: Query<&Team>,
    spatial_grid: Res<SpatialGrid>,
) {
    query
        .iter_mut()
        .for_each(|(entity, transform, battle_stats)| {
            let team = if let Ok(team) = query_team.get(entity) {
                team
            } else {
                return; // 팀 정보를 가져올 수 없으면 건너뜀
            };
            let mut nearby_entities = if let Ok(entity_info_vec) = spatial_grid.query_entities(
                transform.translation.xy(),
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
                let dist_a = query_transform.get(a).map_or(f32::MAX, |t| {
                    transform
                        .translation
                        .xy()
                        .distance_squared(t.translation.xy())
                });
                let dist_b = query_transform.get(b).map_or(f32::MAX, |t| {
                    transform
                        .translation
                        .xy()
                        .distance_squared(t.translation.xy())
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
    query_transform: Query<&Transform>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    attackers
        .iter_mut()
        .for_each(|(entity, mut attack, mut movement, battle_stats)| {
            if attack.attacking == false {
                return;
            }
            let transform = match query_transform.get(entity) {
                Ok(t) => t,
                Err(_) => return, // Transform 컴포넌트가 없으면 스킵
            };
            let dir = if let Ok(target_transform) = query_transform.get(attack.target) {
                (transform.translation.xy() - target_transform.translation.xy()).normalize_or_zero()
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
