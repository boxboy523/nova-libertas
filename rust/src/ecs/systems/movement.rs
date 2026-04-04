use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

const ORDER_MARGIN: f32 = 5.0; // 유닛이 목표 지점에 도달했다고 간주하는 거리

pub fn movement_system(
    mut query: Query<(&mut Transform, &UnitMovement, &FollowingOrder)>,
    orders: Query<&FlowField>,
    grid: Res<FlowGrid>,
    time: Res<Time>,
) {
    let delta = time.delta;
    query
        .par_iter_mut()
        .for_each(|(mut transform, movement, order)| {
            if let Ok(flow_field) = orders.get(order.0) {
                let dir = grid.vector_from_flow_field(flow_field, transform.position);
                let desired_velocity = dir * movement.speed;
                transform.position += desired_velocity * delta;
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

pub fn cleanup_orders_system(
    mut commands: Commands,
    units: Query<&FollowingOrder>,
    orders: Query<Entity, With<MoveOrder>>,
) {
    orders.iter().for_each(|order_entity| {
        let is_using = units
            .iter()
            .any(|following_order| following_order.0 == order_entity);
        if !is_using {
            commands.entity(order_entity).despawn();
        }
    });
}
