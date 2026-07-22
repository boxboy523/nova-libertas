use std::collections::HashSet;

use super::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub transform: Transform,
    pub stats: UnitStats,
    pub team: Team,
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    mut buffer: ResMut<TransformBuffer>,
) {
    let e = commands
        .spawn((
            event.stats,
            event.team,
            Stopped {
                stop_position: event.transform.position,
                in_range: true,
                ..Default::default()
            },
            UnitMovement {
                ..Default::default()
            },
            UnitHp(event.stats.max_hp),
        ))
        .id();
    let transform = buffer.add(event.transform, Some(Vector2::ZERO), Some(event.team), e);
    commands.entity(e).insert(transform);
}

#[derive(Event)]
pub struct MoveOrderEvent {
    pub target_position: Vector2,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
}

pub fn move_order_trigger(
    event: On<MoveOrderEvent>,
    mut commands: Commands,
    mut query: Query<&mut MoveOrder>,
) {
    for mut order in query.iter_mut() {
        order.followers.retain(|e| !event.units.contains(e));
        order.following.retain(|e| !event.units.contains(e));
        order.finished.retain(|e| !event.units.contains(e));
    }
    let new_order = commands
        .spawn((
            MoveOrder {
                target: event.target_position,
                followers: event.units.clone(),
                following: event.units.clone(),
                finished: HashSet::new(),
            },
            FlowField {
                field: Vec::new(),
                goal: event.target_position,
            },
        ))
        .id();
    event.units.iter().for_each(|&unit| {
        commands.entity(unit).insert(Moving {
            order: new_order,
            dist_target_sq: f32::MAX, // 초기값으로 큰 값을 설정
        });
        commands.entity(unit).remove::<Stopped>();
    });
}

pub fn despawn_order_trigger(
    remove: On<Remove, MoveOrder>,
    mut commands: Commands,
    query_order: Query<&MoveOrder>,
    triggered: Query<&DelayedStopTrigger>,
    query_transform: Query<&Transform>,
) {
    if let Ok(order) = query_order.get(remove.entity) {
        for e in &order.followers {
            if triggered.contains(*e) {
                let Ok(transform) = query_transform.get(*e) else {
                    godot_warn!("Failed to get transform for entity {:?}", e);
                    continue;
                };
                commands.entity(*e).remove::<DelayedStopTrigger>();
                commands.entity(*e).remove::<Moving>();
                commands.entity(*e).insert(Stopped {
                    stop_position: transform.position,
                    in_range: true,
                    ..Default::default()
                });
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
    let e = commands.spawn_empty().id();
    let transform = buffer.add(
        Transform {
            position: event.position,
            scale: Vector2::new(1.0, 1.0), // 벽의 스케일은 필요에 따라 조정
            size: event.size.x.max(event.size.y) / 2.0, // 벽의 크기에 따라 size 설정
            t_type: ThingType::Wall,
            ..Default::default()
        },
        None,
        None,
        e,
    );
    commands.entity(e).insert(transform);
}
