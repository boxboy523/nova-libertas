use crate::{combat::component::{Attack, AutoAttack}, movement::{component::{FlowFieldEntity, Moving}, flow_grid::FlowGrid, set_moving}};
use bevy::prelude::*;
use chunk_flow_field::types::FlowField;
use std::collections::HashSet;

#[derive(Event)]
pub struct MoveOrderEvent {
    pub target_position: Vec2,
    pub units: HashSet<Entity>, // 명령을 받을 유닛들
    pub auto_attack: bool,      // 이동 중 자동 공격 여부
}

pub fn move_order_trigger(event: On<MoveOrderEvent>, flow_grid: ResMut<FlowGrid>, mut commands: Commands) {
    println!(
        "MoveOrderEvent received for target position: {:?}, auto_attack: {}",
        event.target_position, event.auto_attack
    );
    let field = flow_grid.gen_flow_field(event.target_position).unwrap_or_else(|err| {
        panic!("Failed to generate flow field: {}", err);
    });
    let new_field = commands
        .spawn((FlowFieldEntity {
            field,
            goal: event.target_position,
        },))
        .id();
    event.units.iter().for_each(|&unit| {
        set_moving(
            &mut commands,
            unit,
            Moving {
                field: new_field,
                dist_target_sq: f32::MAX,
            },
        );
        if event.auto_attack {
            commands.entity(unit).insert(AutoAttack);
        } else {
            commands.entity(unit).remove::<AutoAttack>();
        }
        commands.entity(unit).remove::<Attack>();
    });
}
