use crate::{
    combat::{
        component::{AttackDelivery, AttackImpact},
        event::ImpactEvent,
    },
    prelude::*,
    world3d::Billboard,
};
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
                if dist_sq < battle_stats.range * battle_stats.range {
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
            let mut nearby_entities = if let Ok(entity_info_vec) =
                spatial_grid.query_entities(**position, battle_stats.range, true)
            {
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
    mut attackers: Query<(
        Entity,
        &mut Attack,
        &mut UnitMovement,
        &UnitBattleStats,
        &Team,
        Option<&VisualAnchor>,
    )>,
    query_position: Query<&Position>,
    time: Res<Time>,
    visual: Res<SpriteCatalog>,
) {
    let delta_time = time.delta_secs();
    attackers.iter_mut().for_each(
        |(entity, mut attack, mut movement, battle_stats, team, opt_anchor)| {
            if attack.attacking == false {
                return;
            }
            let position = match query_position.get(entity) {
                Ok(position) => position,
                Err(_) => return, // Position 컴포넌트가 없으면 스킵
            };
            let dir = if let Ok(target_position) = query_position.get(attack.target) {
                (**target_position - **position).normalize_or_zero()
            } else {
                Vec2::ZERO
            };
            movement.preferred_dir = dir;
            // 공격 쿨다운 감소
            attack.cooldown -= delta_time;
            if attack.cooldown <= 0.0 {
                match battle_stats.delivery {
                    AttackDelivery::Instant => match battle_stats.impact {
                        AttackImpact::Single => {
                            commands.trigger(DamageEvent {
                                sender: entity,
                                receiver: attack.target,
                                damage: battle_stats.damage,
                            });
                        }
                        AttackImpact::Area { radius } => {
                            if let Ok(target_position) = query_position.get(attack.target) {
                                commands.trigger(ImpactEvent {
                                    sender: entity,
                                    center: **target_position,
                                    radius,
                                    damage: battle_stats.damage,
                                    team: *team,
                                });
                            }
                        }
                    },
                    AttackDelivery::Projectile { speed, t_type } => {
                        let anchor = opt_anchor.copied().unwrap_or_default();
                        let Some(proj_visual) = visual.sprites.get(&t_type) else {
                            warn!("Projectile sprite not found in catalog");
                            return;
                        };
                        // 투사체 생성
                        let proj = commands
                            .spawn((
                                Projectile {
                                    sender: entity,
                                    target: attack.target,
                                    damage: battle_stats.damage,
                                    speed,
                                    impact: battle_stats.impact,
                                    team: *team,
                                },
                                Transform::from_xyz(position.x, anchor.muzzle.y, position.y),
                            ))
                            .id();
                        spawn_billboard(&mut commands, proj_visual, proj, None);
                    }
                }
                // 공격 후 쿨다운 초기화
                attack.cooldown = battle_stats.cooldown;
            }
        },
    );
}

pub fn projectile_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &Projectile, &mut Transform, &mut Billboard)>,
    query_target: Query<(&Position, Option<&VisualAnchor>)>,
    time: Res<Time>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
) {
    let delta_time = time.delta_secs();
    projectiles
        .iter_mut()
        .for_each(|(entity, projectile, mut transform, mut billboard)| {
            if let Ok((target_position, opt_anchor)) = query_target.get(projectile.target) {
                let anchor = opt_anchor.copied().unwrap_or_default();
                let target_transform =
                    Vec3::new(target_position.x, anchor.hit.y, target_position.y);
                let offset = target_transform - transform.translation;
                let screen_dir = Vec2::new(
                    offset.dot(camera.right().as_vec3()),
                    offset.dot(camera.up().as_vec3()),
                )
                .normalize_or_zero();
                if screen_dir.length_squared() > f32::EPSILON {
                    billboard.roll = screen_dir.to_angle();
                }
                let distance = offset.length();
                let step = projectile.speed * delta_time;
                // 목표 지점에 도달했는지 확인
                if distance <= step {
                    match projectile.impact {
                        AttackImpact::Single => {
                            commands.trigger(DamageEvent {
                                sender: projectile.sender,
                                receiver: projectile.target,
                                damage: projectile.damage,
                            });
                        }
                        AttackImpact::Area { radius } => {
                            commands.trigger(ImpactEvent {
                                sender: projectile.sender,
                                center: Vec2::new(target_position.x, target_position.y),
                                radius,
                                damage: projectile.damage,
                                team: projectile.team,
                            });
                        }
                    }

                    // 투사체 엔티티 제거
                    commands.entity(entity).despawn();
                } else {
                    transform.translation += offset.normalize() * step;
                }
            } else {
                // 목표가 존재하지 않으면 투사체 제거
                commands.entity(entity).despawn();
            }
        });
}
