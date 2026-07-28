use std::collections::HashSet;

use super::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub transform: Transform,
    pub team: Team,
    pub hp: f32, // 유닛의 초기 체력
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    mut buffer: ResMut<TransformBuffer>,
) {
    let Some(stats) = event.transform.t_type.get_unitstats() else {
        return;
    };
    let Some(battle_stats) = event.transform.t_type.get_unit_battle_stats() else {
        return;
    };
    let e = commands
        .spawn((
            event.transform,
            stats,
            battle_stats,
            event.team,
            Stopped {
                stop_position: event.transform.position,
                in_range: true,
                ..Default::default()
            },
            UnitMovement {
                ..Default::default()
            },
            UnitHp(event.hp),
        ))
        .id();
    let transform = buffer.add(
        event.transform,
        Some(Vector2::ZERO),
        Some(event.team),
        e,
        Some(event.hp / stats.max_hp),
    );
    commands.entity(e).insert(transform);
}

#[derive(Event)]
pub struct MoveOrderEvent {
    pub target_position: Vector2,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
}

pub fn move_order_trigger(event: On<MoveOrderEvent>, mut commands: Commands) {
    let new_order = commands
        .spawn((
            MoveOrder {
                target: event.target_position,
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
        commands.entity(unit).remove::<DelayedStopTrigger>();
        commands.entity(unit).remove::<Stopped>();
        commands.entity(unit).remove::<Attacking>();
    });
}

#[derive(Event)]
pub struct AttackOrderEvent {
    pub target: Entity,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
}

pub fn attack_order_trigger(
    event: On<AttackOrderEvent>,
    mut commands: Commands,
    query: Query<&Transform>,
) {
    godot_print!("AttackOrderEvent received for target: {:?}", event.target);
    let last_pos = if let Ok(transform) = query.get(event.target) {
        transform.position
    } else {
        Vector2::ZERO // 대상이 존재하지 않으면 기본값으로 Vector2::ZERO 사용
    };
    let new_order = commands
        .spawn((
            AttackOrder {
                target: event.target,
                last_unit_pos: last_pos,
            },
            FlowField {
                field: Vec::new(),
                goal: Vector2::ZERO, // 공격 명령의 경우 목표 위치는 필요하지 않음
            },
        ))
        .id();
    event.units.iter().for_each(|&unit| {
        commands.entity(unit).insert(Moving {
            order: new_order,
            dist_target_sq: f32::MAX, // 초기값으로 큰 값을 설정
        });
        commands.entity(unit).remove::<DelayedStopTrigger>();
        commands.entity(unit).remove::<Stopped>();
        commands.entity(unit).remove::<Attacking>();
    });
}

pub fn despawn_order_trigger(
    remove: On<Remove, (MoveOrder, AttackOrder)>,
    mut commands: Commands,
    triggered: Query<&DelayedStopTrigger>,
    query: Query<(Entity, &Transform, Option<&Moving>, Option<&Attacking>)>,
) {
    query
        .iter()
        .filter(|(_, _, moving, attacking)| {
            moving.map_or(false, |m| m.order == remove.entity)
                || attacking.map_or(false, |a| a.order == remove.entity)
        })
        .for_each(|(entity, transform, _, _)| {
            commands.entity(entity).remove::<Moving>();
            commands.entity(entity).remove::<Attacking>();
            if triggered.contains(entity) {
                commands.entity(entity).remove::<DelayedStopTrigger>();
            }
            commands.entity(entity).insert(Stopped {
                stop_position: transform.position,
                in_range: true,
                pos_renew_delay: 0.0,
                last_order: Some(remove.entity),
            });
        });
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
        None,
    );
    commands.entity(e).insert(transform);
}

#[derive(Event)]
pub struct DamageEvent {
    pub sender: Entity,
    pub receiver: Entity,
    pub damage: f32,
}

pub fn damage_trigger(
    event: On<DamageEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut UnitHp)>,
) {
    if let Ok((_, mut hp)) = query.get_mut(event.receiver) {
        hp.0 -= event.damage;
        if hp.0 <= 0.0 {
            commands.entity(event.receiver).insert(Dead);
            commands.entity(event.receiver).remove::<Moving>();
            commands.entity(event.receiver).remove::<Attacking>();
            commands.entity(event.receiver).remove::<Stopped>();
        }
    }
}
