use crate::prelude::*;
use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use dodgy_2d::{Agent, AvoidanceOptions};
use std::borrow::Cow;
use std::collections::HashMap;

// 유닛 이동 시스템: UnitMovement 컴포넌트를 가진 엔티티를 이동시키는 시스템
pub fn apply_move_system(
    mut query: Query<(Entity, &mut Position, &mut UnitMovement)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    query
        .par_iter_mut()
        .for_each(|(_, mut position, movement)| {
            if movement.speed < f32::EPSILON {
                return; // 이동할 필요가 없으면 건너뜀
            }
            let direction = movement.dir_vec.normalize_or_zero();
            let speed = movement.speed;
            **position += direction * speed * delta;
        });
}

pub fn smooth_wall_passing_system(
    mut query: Query<(Entity, &Position, &mut UnitMovement, &UnitStats)>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    query
        .par_iter_mut()
        .for_each(|(entity, position, mut movement, stats)| {
            if let CollisionResult::Collided(_, walls) =
                spatial_grid.collision_check(**position, stats.size, Some(&[entity]))
            {
                if !walls.is_empty() {
                    let wall_center = Vec2::new(
                        (walls[0].0 as f32 + 0.5) * spatial_grid.cell_size,
                        (walls[0].1 as f32 + 0.5) * spatial_grid.cell_size,
                    );
                    let wall_vec = **position - wall_center;
                    let normal = if wall_vec.x.abs() > wall_vec.y.abs() * 1.2 {
                        Vec2::new(wall_vec.x.signum(), 0.0)
                    } else if wall_vec.y.abs() > wall_vec.x.abs() * 1.2 {
                        Vec2::new(0.0, wall_vec.y.signum())
                    } else {
                        wall_vec.xy().normalize_or_zero()
                    };
                    movement.dir_vec = normal;
                    movement.speed = stats.max_speed;
                    return;
                }
            }
            if movement.speed < 0.0001 {
                return; // 이동할 필요가 없으면 건너뜀
            }
            let direction = movement.dir_vec.normalize_or_zero();
            let speed = movement.speed;
            let col_pos = **position + direction * (speed * delta);
            match spatial_grid.collision_check(
                col_pos,
                stats.size + OBSTACLE_MARGIN,
                Some(&[entity]),
            ) {
                CollisionResult::NoCollision => {}
                CollisionResult::Collided(_, walls) => {
                    if !walls.is_empty() {
                        let wall_center = Vec2::new(
                            (walls[0].0 as f32 + 0.5) * spatial_grid.cell_size,
                            (walls[0].1 as f32 + 0.5) * spatial_grid.cell_size,
                        );
                        let wall_vec = **position - wall_center;
                        let normal = if wall_vec.x.abs() > wall_vec.y.abs() * 1.2 {
                            Vec2::new(wall_vec.x.signum(), 0.0)
                        } else if wall_vec.y.abs() > wall_vec.x.abs() * 1.2 {
                            Vec2::new(0.0, wall_vec.y.signum())
                        } else {
                            wall_vec.normalize_or_zero()
                        };
                        if normal.dot(direction) < 0.0 {
                            movement.dir_vec =
                                (direction - normal * direction.dot(normal)).normalize_or_zero();
                        }
                    } else {
                    }
                }
                CollisionResult::OutOfBounds => {
                    // 맵 경계 밖으로 나가지 않도록 위치 조정
                    let clamped_x = col_pos
                        .x
                        .clamp(stats.size, spatial_grid.map_size.x - stats.size);
                    let clamped_y = col_pos
                        .y
                        .clamp(stats.size, spatial_grid.map_size.y - stats.size);
                    movement.dir_vec =
                        (Vec2::new(clamped_x, clamped_y) - **position).normalize_or_zero();
                }
            };
        });
}

// 유닛이 목적지 근처에 도달했을 때 멈추는 시스템
pub fn stop_moving_unit_system(
    mut commands: Commands,
    query: Query<
        (Entity, &Position, &Moving, &UnitStats),
        (Without<DelayedStopTrigger>, Without<Attack>),
    >,
    query_stopped: Query<(Entity, &Stopped)>,
    query_fields: Query<&FlowField>,
    spatial_grid: Res<SpatialGrid>,
) {
    let mut stopped_units_map: HashMap<Entity, Vec<Option<Entity>>> = HashMap::new();
    query_stopped.iter().for_each(|(entity, stopped)| {
        if let Some(last_order) = stopped.last_field {
            stopped_units_map
                .entry(last_order)
                .or_insert_with(Vec::new)
                .push(Some(entity));
        }
    });
    query.iter().for_each(|(entity, position, moving, stats)| {
        // 같은 명령을 수행중인 먹춘 유닛과 충돌하면 멈추도록 함
        match spatial_grid.collision_check(
            **position,
            stats.size + STOP_COL_MARGIN,
            Some(&[entity]),
        ) {
            CollisionResult::NoCollision => {}
            CollisionResult::Collided(entity_info_vec, _) => {
                if entity_info_vec.into_iter().any(|e| {
                    stopped_units_map
                        .get(&moving.field)
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
        // 목적지 근처에 도달하면 멈추도록 함
        if let Ok(field) = query_fields.get(moving.field) {
            if position.distance_squared(field.goal) < ORDER_MARGIN * ORDER_MARGIN {
                commands
                    .entity(entity)
                    .insert(DelayedStopTrigger { timer: 0.0 });
            }
        }
    });
}

// 공격 대상이 사라졌을 때 공격을 멈추는 시스템
pub fn stop_attacking_unit_system(
    mut commands: Commands,
    entites: &Entities,
    query: Query<(Entity, &Attack, Option<&AutoAttack>), Without<DelayedStopTrigger>>,
) {
    query.iter().for_each(|(entity, attack, opt_auto)| {
        if !entites.contains(attack.target) {
            if opt_auto.is_some() {
                commands.entity(entity).remove::<Attack>();
            } else {
                commands
                    .entity(entity)
                    .insert(DelayedStopTrigger { timer: STOP_DELAY });
            }
        }
    });
}

// 명령 완료 후 멈춘 유닛이 멈춘 위치로 돌아가는 시스템
pub fn stopped_in_range_system(
    mut query: Query<(&Position, &mut UnitMovement, &mut Stopped)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    query
        .iter_mut()
        .for_each(|(position, mut movement, mut stopped)| {
            let dist_sq = position.distance_squared(stopped.stop_position);
            if dist_sq < RETURN_TO_STOP_MARGIN * RETURN_TO_STOP_MARGIN {
                stopped.in_range = true;
                stopped.out_of_range_time = (stopped.out_of_range_time - delta).max(0.0);
                if movement.preferred_speed <= 0.01 {
                    movement.preferred_speed = 0.0;
                    movement.preferred_dir = Vec2::ZERO;
                }
            } else {
                movement.preferred_dir = (stopped.stop_position - **position).normalize_or_zero();
                stopped.in_range = false;
                stopped.out_of_range_time += delta;
                if stopped.out_of_range_time >= STOP_RENEW_DELAY {
                    stopped.stop_position = **position;
                    stopped.out_of_range_time = 0.0;
                }
            }
        });
}

// DelayedStopTrigger 타이머가 0이 되면 Stopped 컴포넌트를 추가하는 시스템, 유닛이 멈추기 전에 목적지로 좀더 이동하도록 함
pub fn delayed_stop_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedStopTrigger, &Position, &Moving)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    query
        .iter_mut()
        .for_each(|(entity, mut trigger, position, moving)| {
            trigger.timer -= delta;
            if trigger.timer <= 0.0 {
                set_stopped(
                    &mut commands,
                    entity,
                    Stopped {
                        stop_position: **position,
                        in_range: true,
                        out_of_range_time: 0.0,
                        last_field: Some(moving.field),
                    },
                );
            }
        });
}

pub fn update_avoid_resp_system(
    mut query: Query<
        (
            &mut UnitMovement,
            Option<&Moving>,
            Option<&Stopped>,
            Option<&Attack>,
        ),
        Or<(Changed<Moving>, Changed<Stopped>, Changed<Attack>)>,
    >,
) {
    query
        .iter_mut()
        .for_each(|(mut movement, opt_moving, opt_stopped, opt_attack)| {
            if opt_attack.is_some_and(|attack| attack.attacking) {
                movement.avoid_resp = ATTACK_RESP;
            } else if opt_moving.is_some() {
                movement.avoid_resp = MOVE_RESP;
            } else if opt_stopped.is_some() {
                movement.avoid_resp = STOP_RESP;
            } else {
                movement.avoid_resp = STOP_RESP; // 기본값
            }
        });
}

// 유닛 간 분리 시스템: 유닛들이 서로 겹치지 않도록 하는 시스템 (간단한 충돌 회피)
pub fn avoid_system(
    mut query: Query<(Entity, &Position, &mut UnitMovement, &UnitStats)>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    let agents = query
        .iter()
        .map(|(entity, position, movement, stats)| {
            let preferred_velocity =
                movement.preferred_dir.normalize_or_zero() * movement.preferred_speed;
            (
                entity,
                Agent {
                    position: dodgy_2d::Vec2 {
                        x: position.x,
                        y: position.y,
                    },
                    velocity: dodgy_2d::Vec2 {
                        x: preferred_velocity.x,
                        y: preferred_velocity.y,
                    },
                    radius: stats.size,
                    avoidance_responsibility: movement.avoid_resp,
                },
            )
        })
        .collect::<Vec<_>>();
    query
        .par_iter_mut()
        .for_each(|(entity, position, mut movement, stats)| {
            let preferred_velocity =
                movement.preferred_dir.normalize_or_zero() * movement.preferred_speed;
            let agent = Agent {
                position: dodgy_2d::Vec2 {
                    x: position.x,
                    y: position.y,
                },
                velocity: dodgy_2d::Vec2 {
                    x: preferred_velocity.x,
                    y: preferred_velocity.y,
                },
                radius: stats.size,
                avoidance_responsibility: movement.avoid_resp,
            };
            let neighbor_entities = if let Ok(entity_info_vec) =
                spatial_grid.query_entities(**position, stats.size + SEARCH_RADIUS, false)
            {
                entity_info_vec
                    .into_iter()
                    .map(|e| e.entity)
                    .filter(|e| *e != entity)
                    .collect::<Vec<_>>()
            } else {
                warn!("SpatialGrid query_entities failed for entity {:?}", entity);
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
                time.delta_secs(),
                &AvoidanceOptions {
                    obstacle_margin: 0.0,
                    time_horizon: TIME_HORIZON,
                    obstacle_time_horizon: 0.0,
                },
            );
            movement.dir_vec = Vec2::new(val.x, val.y).normalize_or_zero();
            movement.speed = Vec2::new(val.x, val.y).length().min(stats.max_speed);
        });
}

pub fn acceleration_system(
    mut query_moving: Query<
        (&mut UnitMovement, &UnitStats, &Moving, Option<&Attack>),
        Without<Stopped>,
    >,
    mut query_stopped: Query<(&mut UnitMovement, &UnitStats, &Stopped), Without<Moving>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    query_moving
        .par_iter_mut()
        .for_each(|(mut movement, stats, moving, opt_attack)| {
            if let Some(attack) = opt_attack {
                if attack.attacking {
                    movement.preferred_speed -= stats.acceleration * delta * 2.0;
                    movement.preferred_speed = movement.preferred_speed.max(0.0);
                    return;
                }
            }
            let distance = moving.dist_target_sq.sqrt();
            let desired_speed = (2.0 * stats.acceleration * distance)
                .sqrt()
                .min(stats.max_speed);
            let speed_delta = stats.acceleration * delta;
            movement.preferred_speed +=
                (desired_speed - movement.preferred_speed).clamp(-speed_delta, speed_delta);
        });
    query_stopped
        .par_iter_mut()
        .for_each(|(mut movement, stats, stopped)| {
            if stopped.in_range {
                movement.preferred_speed -= stats.acceleration * delta * 2.0;
                movement.preferred_speed = movement.preferred_speed.max(0.0); // 최소 속도 제한
            } else {
                movement.preferred_speed += stats.acceleration * delta;
                movement.preferred_speed = movement.preferred_speed.min(stats.max_speed);
                // 최대 속도 제한
            }
        });
}
