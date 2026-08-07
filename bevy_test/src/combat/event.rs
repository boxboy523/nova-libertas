use crate::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Event)]
pub struct AttackOrderEvent {
    pub target: Entity,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
}

pub fn attack_order_trigger(
    event: On<AttackOrderEvent>,
    mut commands: Commands,
    query: Query<&Position>,
) {
    println!("AttackOrderEvent received for target: {:?}", event.target);
    let last_pos = if let Ok(position) = query.get(event.target) {
        **position
    } else {
        Vec2::ZERO // 대상이 존재하지 않으면 기본값으로 Vector2::ZERO 사용
    };
    let new_field = commands
        .spawn((
            FlowField {
                field: Vec::new(),
                goal: last_pos,
            },
            FieldFollowTarget(event.target),
        ))
        .id();
    event.units.iter().for_each(|&unit| {
        commands.entity(unit).insert(Moving {
            field: new_field,
            dist_target_sq: f32::MAX, // 초기값으로 큰 값을 설정
        });
        commands.entity(unit).remove::<DelayedStopTrigger>();
        commands.entity(unit).remove::<Stopped>();
        commands.entity(unit).remove::<AutoAttack>();
        commands.entity(unit).insert(Attack {
            target: event.target,
            cooldown: 0.0,    // 초기 쿨다운은 0으로 설정
            attacking: false, // 초기에는 공격 중이 아님
        });
    });
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
            commands.entity(event.receiver).remove::<Attack>();
            commands.entity(event.receiver).remove::<Stopped>();
        }
    }
}
