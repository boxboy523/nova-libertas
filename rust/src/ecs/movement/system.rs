use crate::ecs::prelude::*;
use bevy_ecs::entity::Entities;
use bevy_ecs::prelude::*;
use dodgy_2d::{Agent, AvoidanceOptions};
use godot::prelude::*;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

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
    query_fields: Query<&FlowField>,
    flow_grid: Res<FlowGrid>,
) {
    let near_target_margin_squared = NEAR_TARGET_MARGIN * NEAR_TARGET_MARGIN;
    query
        .iter_mut()
        .for_each(|(transform, mut movement, mut moving)| {
            let flow_field = if let Ok(field) = query_fields.get(moving.field) {
                field
            } else {
                return; // FlowField를 찾을 수 없으면 건너뜀
            };
            moving.dist_target_sq = transform.position.distance_squared_to(flow_field.goal);
            movement.preferred_dir = if moving.dist_target_sq < near_target_margin_squared {
                // 목표 지점 근처에 있으면 직선 이동
                (flow_field.goal - transform.position).normalized_or_zero()
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
    query: Query<Entity, With<FlowField>>,
    query_moving: Query<&Moving>,
) {
    let field_using = query_moving
        .iter()
        .map(|moving| moving.field)
        .collect::<HashSet<_>>();
    query.iter().for_each(|entity| {
        if !field_using.contains(&entity) {
            commands.entity(entity).despawn();
        }
    });
}

pub fn stop_moving_unit_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Moving), (Without<DelayedStopTrigger>, Without<Attack>)>,
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
        if let Ok(field) = query_fields.get(moving.field) {
            if transform.position.distance_squared_to(field.goal) < ORDER_MARGIN * ORDER_MARGIN {
                commands
                    .entity(entity)
                    .insert(DelayedStopTrigger { timer: STOP_DELAY });
            }
        }
    });
}

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

pub fn update_flow_field_system(
    mut query_target: Query<(&mut FlowField, &FieldFollowTarget)>,
    query_transform: Query<&Transform>,
    grid: Res<FlowGrid>,
) {
    query_target
        .par_iter_mut()
        .for_each(|(mut flow_field, follow_target)| {
            if let Ok(target_transform) = query_transform.get(follow_target.0) {
                if target_transform.position != flow_field.goal {
                    let last_grid_pos = grid.world_to_grid(flow_field.goal);
                    let new_grid_pos = grid.world_to_grid(target_transform.position);
                    flow_field.goal = target_transform.position;
                    if last_grid_pos != new_grid_pos {
                        flow_field.field = grid
                            .gen_flow_field(flow_field.goal)
                            .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
                    }
                }
            }
        });
}

pub fn flow_field_added_system(
    mut query: Query<&mut FlowField, Added<FlowField>>,
    grid: Res<FlowGrid>,
) {
    query.iter_mut().for_each(|mut flow_field| {
        flow_field.field = grid
            .gen_flow_field(flow_field.goal)
            .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
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
                    last_field: Some(moving.field),
                });
                commands.entity(entity).remove::<DelayedStopTrigger>();
                commands.entity(entity).remove::<Moving>();
                commands.entity(entity).remove::<Attack>();
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
            let neighbor_entities = if let Ok(entity_info_vec) = spatial_grid.query_entities(
                transform.position,
                transform.size + SEARCH_RADIUS,
                false,
            ) {
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

pub fn acceleration_system(
    mut query_moving: Query<
        (&mut UnitMovement, &UnitStats, &Moving, Option<&Attack>),
        Without<Stopped>,
    >,
    mut query_stopped: Query<(&mut UnitMovement, &UnitStats, &Stopped), Without<Moving>>,
    time: Res<Time>,
) {
    query_moving
        .par_iter_mut()
        .for_each(|(mut movement, stats, moving, opt_attack)| {
            if let Some(attack) = opt_attack {
                if attack.attacking {
                    movement.preferred_speed -= stats.acceleration * time.delta * 2.0;
                    movement.preferred_speed = movement.preferred_speed.max(0.0);
                    return;
                }
            }
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
}
