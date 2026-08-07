use crate::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Event)]
pub struct MoveOrderEvent {
    pub target_position: Vec2,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
    pub auto_attack: bool,      // 이동 중 자동 공격 여부
}

pub fn move_order_trigger(event: On<MoveOrderEvent>, mut commands: Commands) {
    println!(
        "MoveOrderEvent received for target position: {:?}, auto_attack: {}",
        event.target_position, event.auto_attack
    );
    let new_field = commands
        .spawn((FlowField {
            field: Vec::new(),
            goal: event.target_position,
        },))
        .id();
    event.units.iter().for_each(|&unit| {
        commands.entity(unit).insert(Moving {
            field: new_field,
            dist_target_sq: f32::MAX, // 초기값으로 큰 값을 설정
        });
        commands.entity(unit).remove::<DelayedStopTrigger>();
        commands.entity(unit).remove::<Stopped>();
        if event.auto_attack {
            commands.entity(unit).insert(AutoAttack);
        } else {
            commands.entity(unit).remove::<AutoAttack>();
        }
        commands.entity(unit).remove::<Attack>();
        commands
            .entity(unit)
            .insert(CurrentAnimation(AnimationKind::Move));
    });
}
