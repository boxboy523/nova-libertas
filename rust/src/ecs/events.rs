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
    pub units: Vec<Entity>, // 명령을 받을 유닛들
}

pub fn move_order_trigger(event: On<MoveOrderEvent>, mut commands: Commands) {
    commands.spawn((
        MoveOrder {
            target: event.target_position,
            followers: event.units.clone(),
        },
        FlowField { field: Vec::new() },
    ));
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
