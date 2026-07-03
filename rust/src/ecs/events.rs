use std::collections::HashSet;

use super::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub transform: Transform,
    pub stats: UnitMovement,
    pub t_type: ThingType,
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    mut buffer: ResMut<TransformBuffer>,
) {
    let e = commands.spawn((event.transform, event.stats)).id();
    let idx = buffer.allocate(event.t_type, e);
    commands.entity(e).insert(TransformID(idx));
}

#[derive(Event)]
pub struct MoveOrderEvent {
    pub target_position: Vector2,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
}

pub fn move_order_trigger(
    event: On<MoveOrderEvent>,
    mut commands: Commands,
    query: Query<Entity, With<MoveOrder>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        MoveOrder {
            target: event.target_position,
            followers: event.units.clone(),
            following: event.units.clone(),
        },
        FlowField {
            field: Vec::new(),
            goal: event.target_position,
        },
    ));
}

pub fn despawn_order_trigger(
    remove: On<Remove, MoveOrder>,
    mut commands: Commands,
    query_order: Query<&MoveOrder>,
    triggered: Query<&DelayedStopTrigger>,
    mut query_movement: Query<&mut UnitMovement>,
) {
    if let Ok(order) = query_order.get(remove.entity) {
        for e in &order.followers {
            if triggered.contains(*e) {
                commands.entity(*e).remove::<DelayedStopTrigger>();
            }
            if let Ok(mut movement) = query_movement.get_mut(*e) {
                movement.moving = false;
                movement.dist_target_sq = f32::MAX;
                movement.dir_vec = Vector2::ZERO;
            }
        }
    }
}

#[derive(Event)]
pub struct SpawnWallEvent {
    pub position: Vector2,
    pub size: Vector2,
}

pub fn spawn_wall_trigger(
    event: On<SpawnWallEvent>,
    mut commands: Commands,
    mut buffer: ResMut<TransformBuffer>,
) {
    let e = commands
        .spawn((Transform {
            position: event.position,
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0), // 벽의 스케일은 필요에 따라 조정
            size: event.size.x.max(event.size.y) / 2.0, // 벽의 크기에 따라 size 설정
        },))
        .id();
    let idx = buffer.allocate(ThingType::Wall, e);
    commands.entity(e).insert(TransformID(idx));
}
