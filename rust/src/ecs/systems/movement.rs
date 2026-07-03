use core::f32;

use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use dodgy_2d::{Agent, AvoidanceOptions};
use godot::prelude::*;
use std::{borrow::Cow, collections::HashSet};

const ORDER_MARGIN: f32 = 10.0; // 유닛이 목표 지점에 도달했다고 간주하는 거리
const NEAR_TARGET_MARGIN: f32 = 60.0; // 유닛이 목표 지점 근처에 있다고 간주하는 거리
const SEARCH_RADIUS: f32 = 50.0; // 주변 유닛 탐색 반경
const OBSTACLE_MARGIN: f32 = 5.0; // 장애물 회피를 위한 마진
const TIME_HORIZON: f32 = 1.0; // 회피 계산 시 예측 시간
const STOP_DELAY: f32 = 0.5; // 명령 완료 후 유닛이 멈추기까지의 지연 시간
const STOP_COL_MARGIN: f32 = 5.0; // 명령 완료한 유닛과 충돌 판정 시 마진
const WALL_PUSH_FACTOR: f32 = 0.1; // 벽과 충돌 시 이동 방향을 얼마나 밀어낼지 결정하는 계수

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
    mut query: Query<(Entity, &Transform, &mut UnitMovement)>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .par_iter_mut()
        .for_each(|(entity, transform, mut movement)| {
            if let CollisionResult::Collided(_, walls) =
                spatial_grid.collision_check(transform.position, transform.size, Some(&[entity]))
            {
                if !walls.is_empty() {
                    godot_print!("wall detected at {:?}, adjusting direction", walls[0]);
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
                    movement.speed = movement.max_speed;
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
    mut commands: Commands,
    mut query: Query<(&mut UnitMovement, &Transform)>,
    orders: Query<(Entity, &FlowField, &MoveOrder)>,
    triggered: Query<&DelayedStopTrigger>,
    flow_grid: Res<FlowGrid>,
    spatial_grid: Res<SpatialGrid>,
) {
    let margin_squared = ORDER_MARGIN * ORDER_MARGIN; // 거리 비교를 위한 제곱값
    let near_target_margin_squared = NEAR_TARGET_MARGIN * NEAR_TARGET_MARGIN;
    orders.iter().for_each(|(entity, flow_field, order)| {
        let finished = order
            .followers
            .difference(&order.following)
            .collect::<HashSet<_>>();
        order.following.iter().for_each(|&unit_entity| {
            if let Ok((mut movement, transform)) = query.get_mut(unit_entity) {
                movement.dist_target_sq = transform.position.distance_squared_to(order.target);
                movement.preferred_dir = if movement.dist_target_sq < near_target_margin_squared {
                    // 목표 지점 근처에 있으면 직선 이동
                    (order.target - transform.position).normalized_or_zero()
                } else if let Some(dir) = // 플로우 필드에서 방향 벡터를 샘플링
                    flow_grid.sample_flow_field(flow_field, transform.position)
                {
                    dir
                } else {
                    Vector2::ZERO
                };
                movement.moving = true;
                if triggered.contains(unit_entity) {
                    return; // 이미 DelayedStopTrigger가 있는 유닛은 건너뜀
                }
                match spatial_grid.collision_check(
                    transform.position,
                    transform.size + STOP_COL_MARGIN,
                    Some(&[unit_entity]),
                ) {
                    CollisionResult::NoCollision => {}
                    CollisionResult::Collided(entity_info_vec, _) => {
                        if entity_info_vec
                            .into_iter()
                            .map(|e| e.entity)
                            .any(|e| finished.contains(&e))
                        {
                            godot_print!(
                                "Unit {:?} collided with finished unit, scheduling stop trigger",
                                unit_entity
                            );
                            commands.entity(unit_entity).insert(DelayedStopTrigger {
                                timer: STOP_DELAY,
                                order: entity,
                            }); // 명령을 완료한 유닛과 충돌하면 0.5초 후에 명령 제거
                        }
                    }
                    CollisionResult::OutOfBounds => {}
                };

                if transform.position.distance_squared_to(order.target) < margin_squared {
                    commands.entity(unit_entity).insert(DelayedStopTrigger {
                        timer: STOP_DELAY,
                        order: entity,
                    }); // 목표 지점에 도달하면 0.5초 후에 명령 제거
                    return;
                }
            }
        });
        if order.followers.is_empty() {
            commands.entity(entity).despawn();
        }
    });
}

pub fn delayed_stop_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DelayedStopTrigger, &mut UnitMovement)>,
    mut orders: Query<&mut MoveOrder>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .iter_mut()
        .for_each(|(entity, mut trigger, mut movement)| {
            trigger.timer -= delta;
            if trigger.timer <= 0.0 {
                godot_print!("DelayedStopTrigger expired for entity {:?}", entity);
                if let Ok(mut order) = orders.get_mut(trigger.order) {
                    order.following.remove(&entity);
                    if order.following.is_empty() {
                        commands.entity(trigger.order).despawn();
                    }
                }
                movement.moving = false;
                movement.dist_target_sq = f32::MAX;
                commands.entity(entity).remove::<DelayedStopTrigger>();
            }
        });
}

// 유닛 간 분리 시스템: 유닛들이 서로 겹치지 않도록 하는 시스템 (간단한 충돌 회피)
pub fn avoid_system(
    mut query: Query<(Entity, &Transform, &mut UnitMovement)>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    let agents = query
        .iter()
        .map(|(entity, transform, movement)| {
            let velocity = movement.preferred_dir.normalized_or_zero() * movement.speed;
            (
                entity,
                Agent {
                    position: dodgy_2d::Vec2 {
                        x: transform.position.x,
                        y: transform.position.y,
                    },
                    velocity: dodgy_2d::Vec2 {
                        x: velocity.x,
                        y: velocity.y,
                    },
                    radius: transform.size,
                    avoidance_responsibility: 1.0,
                },
            )
        })
        .collect::<Vec<_>>();
    query
        .par_iter_mut()
        .for_each(|(entity, transform, mut movement)| {
            let velocity = movement.preferred_dir.normalized_or_zero() * movement.speed;
            let agent = Agent {
                position: dodgy_2d::Vec2 {
                    x: transform.position.x,
                    y: transform.position.y,
                },
                velocity: dodgy_2d::Vec2 {
                    x: velocity.x,
                    y: velocity.y,
                },
                radius: transform.size,
                avoidance_responsibility: 1.0,
            };
            let neighbor_entities = spatial_grid
                .query_entities(transform.position, transform.size + SEARCH_RADIUS)
                .unwrap()
                .into_iter()
                .map(|e| e.entity)
                .filter(|e| *e != entity)
                .collect::<Vec<_>>();
            let neighbors: Vec<Cow<'_, Agent>> = agents
                .iter()
                .filter(|(e, _)| neighbor_entities.contains(e))
                .map(|(_, agent)| Cow::Borrowed(agent))
                .collect::<Vec<_>>();
            let val = agent.compute_avoiding_velocity(
                &neighbors,
                &[],
                dodgy_2d::Vec2 {
                    x: velocity.x,
                    y: velocity.y,
                },
                movement.speed.max(movement.max_speed * 0.3),
                time.delta,
                &AvoidanceOptions {
                    obstacle_margin: 0.0,
                    time_horizon: TIME_HORIZON,
                    obstacle_time_horizon: 0.0,
                },
            );
            movement.dir_vec = Vector2::new(val.x, val.y).normalized_or_zero();
            movement.speed = Vector2::new(val.x, val.y).length().min(movement.max_speed);
        });
}

pub fn update_flow_field_system(
    mut query: Query<(&mut FlowField, &MoveOrder), Changed<MoveOrder>>,
    grid: Res<FlowGrid>,
) {
    query.par_iter_mut().for_each(|(mut flow_field, order)| {
        flow_field.field = grid
            .gen_flow_field(order.target)
            .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
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

pub fn acceleration_system(mut query: Query<&mut UnitMovement>, time: Res<Time>) {
    query.par_iter_mut().for_each(|mut movement| {
        if movement.moving {
            if movement.dist_target_sq < NEAR_TARGET_MARGIN.powi(2) {
                let factor = (movement.dist_target_sq.sqrt() / NEAR_TARGET_MARGIN).clamp(0.5, 1.0);
                movement.speed -= movement.acceleration * time.delta * (2.0 - factor); // 목표 지점 근처에서는 감속
                movement.speed = movement.speed.max(factor * movement.max_speed);
            // 최소 속도 제한
            } else {
                movement.speed += movement.acceleration * time.delta;
                movement.speed = movement.speed.min(movement.max_speed); // 최대 속도 제한
            }
        } else {
            movement.speed -= movement.acceleration * time.delta * 2.0;
            movement.speed = movement.speed.max(0.0); // 최소 속도 제한
        }
    });
}
