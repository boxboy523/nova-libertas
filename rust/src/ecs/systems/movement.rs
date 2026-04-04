use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;

const ORDER_MARGIN: f32 = 5.0; // 유닛이 목표 지점에 도달했다고 간주하는 거리

pub fn movement_system(
    par_commands: ParallelCommands,
    mut query: Query<(&mut Transform, &mut UnitMovement)>,
    mut orders: Query<(Entity, &FlowField, &mut MoveOrder)>,
    grid: Res<FlowGrid>,
    time: Res<Time>,
) {
    let delta = time.delta;
    let margin_squared = ORDER_MARGIN * ORDER_MARGIN; // 거리 비교를 위한 제곱값
    orders
        .iter_mut()
        .for_each(|(entity, flow_field, mut order)| {
            order.followers = order
                .followers
                .iter()
                .filter(|&unit_entity| {
                    if let Ok((mut transform, mut movement)) = query.get_mut(*unit_entity) {
                        let dir = grid.vector_from_flow_field(flow_field, transform.position);
                        movement.moving = true;
                        let velocity = dir * movement.speed;
                        transform.position += velocity * delta;
                        if transform.position.distance_squared_to(order.target) < margin_squared {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();
            if order.followers.is_empty() {
                // 명령이 더 이상 필요 없으면 삭제
                par_commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        });
}

pub fn update_flow_field_system(
    mut query: Query<(&mut FlowField, &MoveOrder), Changed<MoveOrder>>,
    grid: Res<FlowGrid>,
) {
    query.par_iter_mut().for_each(|(mut flow_field, order)| {
        flow_field.field = grid.gen_flow_field(order.target);
    });
}

pub fn acceleration_system(mut query: Query<&mut UnitMovement>) {
    query.par_iter_mut().for_each(|mut movement| {
        if movement.moving {
            movement.speed += movement.acceleration;
            movement.speed = movement.speed.min(100.0); // 최대 속도 제한
        } else {
            movement.speed -= movement.acceleration;
            movement.speed = movement.speed.max(0.0); // 최소 속도 제한
        }
    });
}
