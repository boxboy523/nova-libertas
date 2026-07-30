use crate::ecs::prelude::*;
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

pub fn despawn_order_trigger(
    remove: On<Remove, FlowField>,
    mut commands: Commands,
    triggered: Query<&DelayedStopTrigger>,
    query: Query<(Entity, &Transform, &Moving)>,
) {
    query
        .iter()
        .filter(|(_, _, moving)| moving.field == remove.entity)
        .for_each(|(entity, transform, _)| {
            commands.entity(entity).remove::<Moving>();
            commands.entity(entity).remove::<Attack>();
            if triggered.contains(entity) {
                commands.entity(entity).remove::<DelayedStopTrigger>();
            }
            commands.entity(entity).insert(Stopped {
                stop_position: transform.position,
                in_range: true,
                pos_renew_delay: 0.0,
                last_field: Some(remove.entity),
            });
        });
}
