use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

const ORDER_MARGIN: f32 = 5.0; // 유닛이 목표 지점에 도달했다고 간주하는 거리

// 유닛 이동 시스템: UnitMovement 컴포넌트를 가진 엔티티를 이동시키는 시스템
pub fn apply_move_system(mut query: Query<(&mut Transform, &mut UnitMovement)>, time: Res<Time>) {
    let delta = time.delta;
    query.par_iter_mut().for_each(|(mut transform, movement)| {
        if movement.moving {
            transform.position += movement.dir_vec * movement.speed * delta;
        }
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
                        movement.dir_vec =
                            flow_grid.vector_from_flow_field(flow_field, transform.position);
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

const DAMPING: f32 = 0.3;

// 유닛 간 분리 시스템: 유닛들이 서로 겹치지 않도록 하는 시스템 (간단한 충돌 회피)
pub fn seperation_force_system(
    mut query: Query<(Entity, &mut Transform)>,
    spatial_grid: Res<SpatialGrid>,
) {
    let to_move = query
        .iter()
        .filter_map(|(e, transform)| {
            let nearby_entities = spatial_grid.query_entities(transform.position, 40.0); // 일정 반경 내의 엔티티 조회
            for other_entity in nearby_entities {
                if let Ok((_, other_transform)) = query.get(other_entity) {
                    let to_other = transform.position - other_transform.position;
                    let distance_squared = to_other.length_squared();
                    if distance_squared < (transform.size + other_transform.size).powi(2) {
                        return Some((e, to_other * DAMPING)); // 가까운 만큼 반발격 적용
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>();
    for (entity, force) in to_move {
        if let Ok((_, mut transform)) = query.get_mut(entity) {
            transform.position += force; // 분리 힘 적용
        }
    }
}

pub fn update_flow_field_system(
    mut query: Query<(&mut FlowField, &MoveOrder), Changed<MoveOrder>>,
    grid: Res<FlowGrid>,
) {
    query.par_iter_mut().for_each(|(mut flow_field, order)| {
        flow_field.field = grid.gen_flow_field(order.target);
    });
}

pub fn update_spatial_grid_system(
    object: Query<(Entity, &Transform)>,
    mut grid: ResMut<SpatialGrid>,
) {
    grid.clear();
    object.iter().for_each(|(entity, transform)| {
        grid.add_entity(entity, transform.position);
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
