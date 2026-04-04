use super::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub transform: Transform,
    pub stats: UnitMovement,
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    mut buffer: ResMut<TransformBuffer>,
) {
    let id = buffer.allocate();
    commands.spawn((
        event.transform,
        event.stats,
        TransformID(id), // 새 유닛에 고유한 TransformID 할당
    ));
}

#[derive(Event)]
pub struct MoveOrderEvent {
    pub target_position: Vector2,
    pub units: Vec<Entity>, // 명령을 받을 유닛들
}

pub fn move_order_trigger(event: On<MoveOrderEvent>, mut commands: Commands) {
    let order = commands
        .spawn((
            MoveOrder {
                target: event.target_position,
            },
            FlowField { field: Vec::new() },
        ))
        .id();
    for unit in event.units.iter() {
        commands.entity(*unit).insert(FollowingOrder(order));
    }
}
