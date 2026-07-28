use core::f32;

use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use dodgy_2d::{Agent, AvoidanceOptions};
use godot::prelude::*;
use std::{borrow::Cow, collections::HashMap};

const ORDER_MARGIN: f32 = 10.0; // 유닛이 목표 지점에 도달했다고 간주하는 거리
const NEAR_TARGET_MARGIN: f32 = 60.0; // 유닛이 목표 지점 근처에 있다고 간주하는 거리
const SEARCH_RADIUS: f32 = 50.0; // 주변 유닛 탐색 반경
const OBSTACLE_MARGIN: f32 = 5.0; // 장애물 회피를 위한 마진
const TIME_HORIZON: f32 = 1.0; // 회피 계산 시 예측 시간
const STOP_DELAY: f32 = 0.5; // 명령 완료 후 유닛이 멈추기까지의 지연 시간
const STOP_COL_MARGIN: f32 = 5.0; // 명령 완료한 유닛과 충돌 판정 시 마진S
const RETURN_TO_STOP_MARGIN: f32 = 20.0; // 명령 완료 후 유닛이 멈춘 위치로 돌아갈 때의 마진
const STOP_RENEW_DELAY: f32 = 1.0; // 멈춘 위치 갱신 지연 시간

const MOVING_RESP: f32 = 0.5;
const STOP_RESP: f32 = 2.0;

// 유닛 이동 시스템: UnitMovement 컴포넌트를 가진 엔티티를 이동시키는 시스템
pub fn apply_move_system(
    mut query: Query<(Entity, &mut Transform, &mut UnitMovement)>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .par_iter_mut()
        .for_each(|(_, mut transform, movement)| {
            if movement.speed < f32::EPSILON {
                return; // 이동할 필요가 없으면 건너뜀
            }
            let direction = movement.dir_vec.normalized_or_zero();
            let speed = movement.speed;
            transform.position += direction * speed * delta;
        });
}

pub fn smooth_wall_passing_system(
    mut query: Query<(Entity, &Transform, &mut UnitMovement, &UnitStats)>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .par_iter_mut()
        .for_each(|(entity, transform, mut movement, stats)| {
            if let CollisionResult::Collided(_, walls) =
                spatial_grid.collision_check(transform.position, transform.size, Some(&[entity]))
            {
                if !walls.is_empty() {
                    let wall_center = Vector2::new(
                        (walls[0].0 as f32 + 0.5) * spatial_grid.cell_size,
                        (walls[0].1 as f32 + 0.5) * spatial_grid.cell_size,
                    );
                    let wall_vec = transform.position - wall_center;
                    let normal = if wall_vec.x.abs() > wall_vec.y.abs() * 1.2 {
                        Vector2::new(wall_vec.x.signum(), 0.0)
                    } else if wall_vec.y.abs() > wall_vec.x.abs() * 1.2 {
                        Vector2::new(0.0, wall_vec.y.signum())
                    } else {
                        wall_vec.normalized_or_zero()
                    };
                    movement.dir_vec = normal;
                    movement.speed = stats.max_speed;
                    return;
                }
            }
            if movement.speed < 0.0001 {
                return; // 이동할 필요가 없으면 건너뜀
            }
            let direction = movement.dir_vec.normalized_or_zero();
            let speed = movement.speed;
            let col_pos = transform.position + direction * (speed * delta);
            match spatial_grid.collision_check(
                col_pos,
                transform.size + OBSTACLE_MARGIN,
                Some(&[entity]),
            ) {
                CollisionResult::NoCollision => {}
                CollisionResult::Collided(_, walls) => {
                    if !walls.is_empty() {
                        let wall_center = Vector2::new(
                            (walls[0].0 as f32 + 0.5) * spatial_grid.cell_size,
                            (walls[0].1 as f32 + 0.5) * spatial_grid.cell_size,
                        );
                        let wall_vec = transform.position - wall_center;
                        let normal = if wall_vec.x.abs() > wall_vec.y.abs() * 1.2 {
                            Vector2::new(wall_vec.x.signum(), 0.0)
                        } else if wall_vec.y.abs() > wall_vec.x.abs() * 1.2 {
                            Vector2::new(0.0, wall_vec.y.signum())
                        } else {
                            wall_vec.normalized_or_zero()
                        };
                        if normal.dot(direction) < 0.0 {
                            movement.dir_vec =
                                (direction - normal * direction.dot(normal)).normalized_or_zero();
                        }
                    } else {
                    }
                }
                CollisionResult::OutOfBounds => {
                    // 맵 경계 밖으로 나가지 않도록 위치 조정
                    let clamped_x = col_pos
                        .x
                        .clamp(transform.size, spatial_grid.map_size.x - transform.size);
                    let clamped_y = col_pos
                        .y
                        .clamp(transform.size, spatial_grid.map_size.y - transform.size);
                    movement.dir_vec = (Vector2::new(clamped_x, clamped_y) - transform.position)
                        .normalized_or_zero();
                }
            };
        });
}

// 명령 처리 시스템: 명령을 받은 유닛들을 FlowField를 따라 방향을 업데이트하는 시스템
pub fn flow_movement_system(
    mut query: Query<(&Transform, &mut UnitMovement, &mut Moving)>,
    move_orders: Query<(&FlowField, &MoveOrder), Without<AttackOrder>>,
    attack_orders: Query<(&FlowField, &AttackOrder), Without<MoveOrder>>,
    flow_grid: Res<FlowGrid>,
) {
    let near_target_margin_squared = NEAR_TARGET_MARGIN * NEAR_TARGET_MARGIN;
    query
        .iter_mut()
        .for_each(|(transform, mut movement, mut moving)| {
            let (flow_field, target) =
                if let Ok((flow_field, order)) = move_orders.get(moving.order) {
                    (flow_field, order.target)
                } else if let Ok((flow_field, attack_order)) = attack_orders.get(moving.order) {
                    (flow_field, attack_order.last_unit_pos)
                } else {
                    return;
                };
            moving.dist_target_sq = transform.position.distance_squared_to(target);
            movement.preferred_dir = if moving.dist_target_sq < near_target_margin_squared {
                // 목표 지점 근처에 있으면 직선 이동
                (target - transform.position).normalized_or_zero()
            } else if let Some(dir) = // 플로우 필드에서 방향 벡터를 샘플링
                flow_grid.sample_flow_field(flow_field, transform.position)
            {
                dir
            } else {
                Vector2::ZERO
            };
        });
}

pub fn remove_empty_orders_system(
    mut commands: Commands,
    query: Query<Entity, Or<(With<MoveOrder>, With<AttackOrder>)>>,
    query_moving: Query<&Moving>,
    query_attacking: Query<&Attacking>,
) {
    query.iter().for_each(|entity| {
        let has_following_units = query_moving.iter().any(|moving| moving.order == entity)
            || query_attacking
                .iter()
                .any(|attacking| attacking.order == entity);
        if !has_following_units {
            commands.entity(entity).despawn();
        }
    });
}

pub fn stop_unit_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Moving), Without<DelayedStopTrigger>>,
    query_stopped: Query<(Entity, &Stopped)>,
    orders: Query<&MoveOrder>,
    spatial_grid: Res<SpatialGrid>,
) {
    let mut stopped_units_map: HashMap<Entity, Vec<Option<Entity>>> = HashMap::new();
    query_stopped.iter().for_each(|(entity, stopped)| {
        if let Some(last_order) = stopped.last_order {
            stopped_units_map
                .entry(last_order)
                .or_insert_with(Vec::new)
                .push(Some(entity));
        }
    });
    query.iter().for_each(|(entity, transform, moving)| {
        match spatial_grid.collision_check(
            transform.position,
            transform.size + STOP_COL_MARGIN,
            Some(&[entity]),
        ) {
            CollisionResult::NoCollision => {}
            CollisionResult::Collided(entity_info_vec, _) => {
                if entity_info_vec.into_iter().any(|e| {
                    stopped_units_map
                        .get(&moving.order)
                        .map_or(false, |stopped_units| {
                            stopped_units.iter().any(|&stopped_entity| {
                                stopped_entity
                                    .map_or(false, |stopped_entity| stopped_entity == e.entity)
                            })
                        })
                }) {
                    commands
                        .entity(entity)
                        .insert(DelayedStopTrigger { timer: STOP_DELAY });
                }
            }
            CollisionResult::OutOfBounds => {}
        }
        if let Ok(order) = orders.get(moving.order) {
            if transform.position.distance_squared_to(order.target) < ORDER_MARGIN * ORDER_MARGIN {
                commands
                    .entity(entity)
                    .insert(DelayedStopTrigger { timer: STOP_DELAY });
            }
        }
    });
}

pub fn move_or_attack_system(
    mut commands: Commands,
    query_moving: Query<(Entity, &Transform, &Moving, &UnitBattleStats)>,
    query_attacking: Query<(Entity, &Transform, &Attacking, &UnitBattleStats)>,
    query_attack: Query<&AttackOrder>,
    query_transform: Query<&Transform>,
) {
    query_moving
        .iter()
        .for_each(|(entity, transform, moving, battle_stats)| {
            if let Ok(attack_order) = query_attack.get(moving.order) {
                if let Ok(target_transform) = query_transform.get(attack_order.target) {
                    let dist_sq = transform
                        .position
                        .distance_squared_to(target_transform.position);
                    if dist_sq < battle_stats.attack_range * battle_stats.attack_range {
                        commands.entity(entity).insert(Attacking {
                            order: moving.order,
                            cooldown: 0.0,
                            dist_target_sq: dist_sq,
                        });
                        commands.entity(entity).remove::<Moving>();
                    }
                } else {
                    commands.entity(entity).remove::<Moving>();
                    commands.entity(entity).insert(Stopped {
                        stop_position: transform.position,
                        in_range: true,
                        pos_renew_delay: 0.0,
                        last_order: None,
                    });
                    commands.entity(moving.order).despawn(); // 공격 대상이 없으면 명령 제거
                }
            }
        });
    query_attacking
        .iter()
        .for_each(|(entity, transform, attacking, battle_stats)| {
            if let Ok(attack_order) = query_attack.get(attacking.order) {
                if let Ok(target_transform) = query_transform.get(attack_order.target) {
                    let dist_sq = transform
                        .position
                        .distance_squared_to(target_transform.position);
                    if dist_sq > battle_stats.attack_range * battle_stats.attack_range {
                        commands.entity(entity).insert(Moving {
                            order: attacking.order,
                            dist_target_sq: dist_sq,
                        });
                        commands.entity(entity).remove::<Attacking>();
                    }
                } else {
                    commands.entity(entity).remove::<Attacking>();
                    commands.entity(entity).insert(Stopped {
                        stop_position: transform.position,
                        in_range: true,
                        pos_renew_delay: 0.0,
                        last_order: None,
                    });
                    commands.entity(attacking.order).despawn(); // 공격 대상이 없으면 명령 제거
                }
            }
        });
}

pub fn delayed_stop_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedStopTrigger, &Transform, &Moving)>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .iter_mut()
        .for_each(|(entity, mut trigger, transform, moving)| {
            trigger.timer -= delta;
            if trigger.timer <= 0.0 {
                commands.entity(entity).insert(Stopped {
                    stop_position: transform.position,
                    in_range: true,
                    pos_renew_delay: 0.0,
                    last_order: Some(moving.order),
                });
                commands.entity(entity).remove::<DelayedStopTrigger>();
                commands.entity(entity).remove::<Moving>();
            }
        });
}

// 유닛 간 분리 시스템: 유닛들이 서로 겹치지 않도록 하는 시스템 (간단한 충돌 회피)
pub fn avoid_system(
    mut query: Query<(
        Entity,
        &Transform,
        &mut UnitMovement,
        &UnitStats,
        Option<&Moving>,
    )>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    let agents = query
        .iter()
        .map(|(entity, transform, movement, _, opt_moving)| {
            let preferred_velocity =
                movement.preferred_dir.normalized_or_zero() * movement.preferred_speed;
            (
                entity,
                Agent {
                    position: dodgy_2d::Vec2 {
                        x: transform.position.x,
                        y: transform.position.y,
                    },
                    velocity: dodgy_2d::Vec2 {
                        x: preferred_velocity.x,
                        y: preferred_velocity.y,
                    },
                    radius: transform.size,
                    avoidance_responsibility: if opt_moving.is_some() {
                        MOVING_RESP
                    } else {
                        STOP_RESP
                    },
                },
            )
        })
        .collect::<Vec<_>>();
    query
        .par_iter_mut()
        .for_each(|(entity, transform, mut movement, stats, opt_moving)| {
            let preferred_velocity =
                movement.preferred_dir.normalized_or_zero() * movement.preferred_speed;
            let agent = Agent {
                position: dodgy_2d::Vec2 {
                    x: transform.position.x,
                    y: transform.position.y,
                },
                velocity: dodgy_2d::Vec2 {
                    x: preferred_velocity.x,
                    y: preferred_velocity.y,
                },
                radius: transform.size,
                avoidance_responsibility: if opt_moving.is_some() {
                    MOVING_RESP
                } else {
                    STOP_RESP
                },
            };
            let neighbor_entities = if let Ok(entity_info_vec) =
                spatial_grid.query_entities(transform.position, transform.size + SEARCH_RADIUS)
            {
                entity_info_vec
                    .into_iter()
                    .map(|e| e.entity)
                    .filter(|e| *e != entity)
                    .collect::<Vec<_>>()
            } else {
                godot_warn!("SpatialGrid query_entities failed for entity {:?}", entity);
                vec![]
            };
            let neighbors: Vec<Cow<'_, Agent>> = agents
                .iter()
                .filter(|(e, _)| neighbor_entities.contains(e))
                .map(|(_, agent)| Cow::Borrowed(agent))
                .collect::<Vec<_>>();
            let val = agent.compute_avoiding_velocity(
                &neighbors,
                &[],
                dodgy_2d::Vec2 {
                    x: preferred_velocity.x,
                    y: preferred_velocity.y,
                },
                stats.max_speed,
                time.delta,
                &AvoidanceOptions {
                    obstacle_margin: 0.0,
                    time_horizon: TIME_HORIZON,
                    obstacle_time_horizon: 0.0,
                },
            );
            movement.dir_vec = Vector2::new(val.x, val.y).normalized_or_zero();
            movement.speed = Vector2::new(val.x, val.y).length().min(stats.max_speed);
        });
}

pub fn update_flow_field_system(
    mut query_move: Query<(&mut FlowField, &MoveOrder), (Changed<MoveOrder>, Without<AttackOrder>)>,
    mut query_attack: Query<(&mut FlowField, &mut AttackOrder), Without<MoveOrder>>,
    query_transform: Query<&Transform>,
    grid: Res<FlowGrid>,
) {
    query_move
        .par_iter_mut()
        .for_each(|(mut flow_field, order)| {
            flow_field.field = grid
                .gen_flow_field(order.target)
                .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
        });
    query_attack
        .par_iter_mut()
        .for_each(|(mut flow_field, mut attack_order)| {
            if let Ok(target_transform) = query_transform.get(attack_order.target) {
                let new_pos = grid.world_to_grid(target_transform.position);
                if flow_field.field.is_empty()
                    || new_pos != grid.world_to_grid(attack_order.last_unit_pos)
                {
                    flow_field.field = grid
                        .gen_flow_field(target_transform.position)
                        .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
                }
                attack_order.last_unit_pos = target_transform.position;
            }
        });
}

pub fn update_spatial_grid_system(
    object: Query<(Entity, &Transform), With<UnitMovement>>,
    mut grid: ResMut<SpatialGrid>,
) {
    grid.clear();
    object.iter().for_each(|(entity, transform)| {
        grid.add_entity(entity, transform.position, transform.size)
            .ok();
    });
}

pub fn stopped_in_range_system(
    mut query: Query<(&Transform, &mut UnitMovement, &mut Stopped)>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .iter_mut()
        .for_each(|(transform, mut movement, mut stopped)| {
            if transform
                .position
                .distance_squared_to(stopped.stop_position)
                < RETURN_TO_STOP_MARGIN * RETURN_TO_STOP_MARGIN
            {
                movement.preferred_dir =
                    (stopped.stop_position - transform.position).normalized_or_zero();
                stopped.in_range = true;
                stopped.pos_renew_delay += delta;
                if stopped.pos_renew_delay >= STOP_RENEW_DELAY {
                    stopped.stop_position = transform.position;
                    stopped.pos_renew_delay = 0.0;
                }
            } else {
                movement.preferred_dir =
                    (stopped.stop_position - transform.position).normalized_or_zero();
                stopped.in_range = false;
            }
        });
}

pub fn acceleration_system(
    mut query_moving: Query<
        (&mut UnitMovement, &UnitStats, &Moving),
        (Without<Stopped>, Without<Attacking>),
    >,
    mut query_stopped: Query<
        (&mut UnitMovement, &UnitStats, &Stopped),
        (Without<Moving>, Without<Attacking>),
    >,
    mut query_attacking: Query<
        (&mut UnitMovement, &UnitStats),
        (With<Attacking>, Without<Moving>, Without<Stopped>),
    >,
    time: Res<Time>,
) {
    query_moving
        .par_iter_mut()
        .for_each(|(mut movement, stats, moving)| {
            if moving.dist_target_sq < NEAR_TARGET_MARGIN.powi(2) {
                let factor = (moving.dist_target_sq.sqrt() / NEAR_TARGET_MARGIN).clamp(0.5, 1.0);
                movement.preferred_speed -= stats.acceleration * time.delta * (2.0 - factor); // 목표 지점 근처에서는 감속
                movement.preferred_speed = movement.preferred_speed.max(factor * stats.max_speed);
            // 최소 속도 제한
            } else {
                movement.preferred_speed += stats.acceleration * time.delta;
                movement.preferred_speed = movement.preferred_speed.min(stats.max_speed);
                // 최대 속도 제한
            }
        });
    query_stopped
        .par_iter_mut()
        .for_each(|(mut movement, stats, stopped)| {
            if stopped.in_range {
                movement.preferred_speed -= stats.acceleration * time.delta * 2.0;
                movement.preferred_speed = movement.preferred_speed.max(0.0); // 최소 속도 제한
            } else {
                movement.preferred_speed += stats.acceleration * time.delta;
                movement.preferred_speed = movement.preferred_speed.min(stats.max_speed);
                // 최대 속도 제한
            }
        });
    query_attacking
        .par_iter_mut()
        .for_each(|(mut movement, stats)| {
            movement.preferred_speed -= stats.acceleration * time.delta * 2.0;
            movement.preferred_speed = movement.preferred_speed.max(0.0);
        });
}
