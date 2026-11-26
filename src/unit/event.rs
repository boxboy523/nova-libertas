use crate::{
    combat::component::UnitHp,
    movement::component::{Stopped, UnitMovement},
    thing::{ThingCatalog, ThingId},
    unit::component::{Position, Team},
    visual::{SpriteCatalog, data::{AnimationKind, CurrentAnimation}},
    world3d::spawn_billboard,
};
use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub position: Vec2,
    pub id: ThingId,
    pub team: Team,
    pub hp: f32, // 유닛의 초기 체력
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    catalog: Res<ThingCatalog>,
    sprite_catalog: Res<SpriteCatalog>,
) {
    let Some(info) = catalog.get_info(event.id) else {
        warn!("ThingType {:?} not found in catalog", event.id);
        return;
    };
    if info.unit_stats.is_none() || info.battle_stats.is_none() {
        warn!(
            "ThingType {:?} does not have unit_stats or battle_stats",
            event.id
        );
        return;
    }
    let e = commands
        .spawn((
            Position(event.position),
            Transform::from_translation(Vec3::new(event.position.x, 0.0, event.position.y)),
            info.unit_stats.unwrap(),
            info.battle_stats.unwrap(),
            event.team,
            Stopped {
                stop_position: event.position,
                in_range: true,
                ..Default::default()
            },
            UnitMovement::default(),
            UnitHp {
                current: event.hp,
                max: event.hp,
            },
            event.id,
            CurrentAnimation(AnimationKind::Stand)
        ))
        .id();
    let Some(unit_visual) = sprite_catalog.sprites.get(&event.id) else {
        warn!(
            "UnitSprite for ThingType {:?} not found in catalog",
            event.id
        );
        return;
    };
    spawn_billboard(&mut commands, unit_visual, e, Some(event.team));
    commands.entity(e).insert(Transform::from_translation(
        Vec3::new(event.position.x, 0.0, event.position.y)),
    );
    if let Some(anchor) = unit_visual.anchor {
        commands.entity(e).insert(anchor);
    }
}
