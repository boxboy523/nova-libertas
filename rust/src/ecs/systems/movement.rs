use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

const ORDER_MARGIN: f32 = 10.0; // 유닛이 목표 지점에 도달했다고 간주하는 거리
const SEP_WEIGHT: f32 = 1.0; // 분리 힘의 가중치
const MAX_SEP_FORCE: f32 = 50.0; // 최대 분리 힘
const SEP_DIST: f32 = 20.0; // 유닛 간 최소 거리
const SEP_BOOST: f32 = 1.5; // 분리 힘이 최대일 때 속도에 곱해지는 보정값
const SEARCH_RADIUS: f32 = 50.0; // 주변 유닛 탐색 반경

// 유닛 이동 시스템: UnitMovement 컴포넌트를 가진 엔티티를 이동시키는 시스템
pub fn apply_move_system(
    mut query: Query<(Entity, &mut Transform, &mut UnitMovement)>,
    time: Res<Time>,
    spatial_grid: Res<SpatialGrid>,
) {
    let delta = time.delta;
    query
        .par_iter_mut()
        .for_each(|(entity, mut transform, mut movement)| {
            if movement.speed < f32::EPSILON && movement.seperation_force == Vector2::ZERO {
                return; // 이동할 필요가 없으면 건너뜀
            }
            let direction = if movement.moving {
                (movement.dir_vec + movement.seperation_force * SEP_WEIGHT).normalized_or_zero()
            } else {
                movement.seperation_force.normalized_or_zero()
            };
            let speed = if movement.moving {
                ((movement.speed / movement.max_speed
                    + (movement.seperation_force.length() / MAX_SEP_FORCE).min(1.0))
                    * movement.max_speed)
                    .min(movement.max_speed * SEP_BOOST) // 최대 속도보다 약간 빠르게 허용
            } else {
                (movement.seperation_force.length() / MAX_SEP_FORCE * 10.0).min(1.0)
                    * movement.max_speed
            };
            godot_print!(
                "direction: {:?}, speed: {}, seperation_force: {:?}, delta: {}",
                direction,
                speed,
                movement.seperation_force,
                delta
            );
            movement.seperation_force = Vector2::ZERO;
            let next_pos = transform.position + direction * speed * delta;
            let col_pos = transform.position + direction * (0.1 + speed * delta);
            let sign_x = direction.x.signum();
            let sign_y = direction.y.signum();
            let move_x = transform.position + Vector2::new(sign_x * speed * delta, 0.0);
            let move_y = transform.position + Vector2::new(0.0, speed * delta * sign_y);
            let col_x = transform.position + Vector2::new(sign_x * (0.1 + speed * delta), 0.0);
            let col_y = transform.position + Vector2::new(0.0, sign_y * (0.1 + speed * delta));
            let x_free = spatial_grid.collision_check(col_x, transform.size, Some(&[entity]))
                == CollisionResult::NoCollision;
            let y_free = spatial_grid.collision_check(col_y, transform.size, Some(&[entity]))
                == CollisionResult::NoCollision;
            godot_print!("x_free: {}, y_free: {}", x_free, y_free);
            godot_print!(
                "collision check result: {:?}",
                spatial_grid.collision_check(next_pos, transform.size, Some(&[entity]))
            );
            match spatial_grid.collision_check(col_pos, transform.size, Some(&[entity])) {
                CollisionResult::NoCollision => transform.position = next_pos,
                CollisionResult::CollidedWall(_) => {
                    if x_free {
                        transform.position = move_x;
                    } else if y_free {
                        transform.position = move_y;
                    }
                }
                CollisionResult::Collided(_) => {
                    // if x_free {
                    //     transform.position = move_x;
                    // } else if y_free {
                    //     transform.position = move_y;
                    // }
                }
                CollisionResult::OutOfBounds => {
                    // 맵 경계 밖으로 나가지 않도록 위치 조정
                    let clamped_x = next_pos
                        .x
                        .clamp(transform.size, spatial_grid.map_size.x - transform.size);
                    let clamped_y = next_pos
                        .y
                        .clamp(transform.size, spatial_grid.map_size.y - transform.size);
                    transform.position = Vector2::new(clamped_x, clamped_y);
                }
            };
        });
}

// 명령 처리 시스템: 명령을 받은 유닛들을 FlowField를 따라 방향을 업데이트하는 시스템
pub fn flow_movement_system(
    mut commands: Commands,
    mut query: Query<(&mut UnitMovement, &Transform)>,
    mut orders: Query<(Entity, &FlowField, &mut MoveOrder)>,
    flow_grid: Res<FlowGrid>,
) {
    let margin_squared = ORDER_MARGIN * ORDER_MARGIN; // 거리 비교를 위한 제곱값
    orders
        .iter_mut()
        .for_each(|(entity, flow_field, mut order)| {
            order.followers = order
                .followers
                .iter()
                .filter(|&unit_entity| {
                    if let Ok((mut movement, transform)) = query.get_mut(*unit_entity) {
                        movement.dir_vec = if let Some(dir) =
                            flow_grid.sample_flow_field(flow_field, transform.position)
                        {
                            if dir == Vector2::ZERO {
                                // it means unit is on target grid cell
                                (order.target - transform.position).normalized_or_zero()
                            } else {
                                dir
                            }
                        } else {
                            Vector2::ZERO
                        };
                        movement.moving = true;
                        if transform.position.distance_squared_to(order.target) < margin_squared {
                            movement.moving = false; // 목표 지점에 도달하면 이동 중지
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();
            if order.followers.is_empty() {
                commands.entity(entity).despawn();
            }
        });
}

// 유닛 간 분리 시스템: 유닛들이 서로 겹치지 않도록 하는 시스템 (간단한 충돌 회피)
pub fn seperation_force_system(
    mut query: Query<(Entity, &Transform, &mut UnitMovement)>,
    spatial_grid: Res<SpatialGrid>,
) {
    let to_move = query
        .iter()
        .filter_map(|(e, transform, movement)| {
            let nearby_entities = spatial_grid
                .query_entities(
                    transform.position,
                    transform.size + SEP_DIST + SEARCH_RADIUS,
                )
                .unwrap_or(Vec::new());
            let mut total_force = Vector2::ZERO;
            for other_entity in nearby_entities {
                if let Ok((_, other_transform, other_movement)) = query.get(other_entity.entity) {
                    if movement.moving && !other_movement.moving {
                        continue;
                    }
                    let to_other = transform.position - other_transform.position;
                    let distance = to_other.length();
                    if distance < (transform.size + other_transform.size + SEP_DIST) {
                        let overlap = (transform.size + other_transform.size + SEP_DIST) - distance;
                        let force = to_other.normalized_or_zero() * overlap;
                        total_force += force;
                    }
                }
            }
            if total_force != Vector2::ZERO {
                Some((e, total_force))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for (entity, force) in to_move {
        if let Ok((_, _, mut unit_movement)) = query.get_mut(entity) {
            unit_movement.seperation_force += force
        }
    }
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

pub fn acceleration_system(mut query: Query<&mut UnitMovement>) {
    query.par_iter_mut().for_each(|mut movement| {
        if movement.moving {
            movement.speed += movement.acceleration;
            movement.speed = movement.speed.min(movement.max_speed); // 최대 속도 제한
        } else {
            movement.speed -= movement.acceleration;
            movement.speed = movement.speed.max(0.0); // 최소 속도 제한
        }
    });
}
