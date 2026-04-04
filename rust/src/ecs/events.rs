use super::prelude::*;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub transform: Transform,
    pub stats: UnitStats,
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
