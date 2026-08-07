use crate::prelude::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub position: Vec2,
    pub t_type: ThingType,
    pub team: Team,
    pub hp: f32, // 유닛의 초기 체력
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    catalog: Res<ThingCatalog>,
    sprite_catalog: Res<SpriteCatalog>,
) {
    let Some(info) = catalog.get_info(event.t_type) else {
        warn!("ThingType {:?} not found in catalog", event.t_type);
        return;
    };
    if info.unit_stats.is_none() || info.battle_stats.is_none() {
        warn!(
            "ThingType {:?} does not have unit_stats or battle_stats",
            event.t_type
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
            UnitHp(event.hp),
            event.t_type,
        ))
        .id();
    let Some(unit_visual) = sprite_catalog.sprites.get(&event.t_type) else {
        warn!(
            "UnitSprite for ThingType {:?} not found in catalog",
            event.t_type
        );
        return;
    };
    spawn_billboard(&mut commands, unit_visual, e);
    commands
        .entity(e)
        .insert(CurrentAnimation(AnimationKind::Stand));
}

#[derive(Event)]
pub struct SpawnWallEvent {
    pub position: Vec2,
}

pub fn spawn_wall_trigger(
    event: On<SpawnWallEvent>,
    mut commands: Commands,
    sprite_catalog: Res<SpriteCatalog>,
) {
    let e = commands.spawn_empty().id();
    let transform = Transform::from_translation(Vec3::new(event.position.x, 0.0, event.position.y));
    let Some(visual) = sprite_catalog.sprites.get(&ThingType::Wall) else {
        warn!("UnitSprite for ThingType Wall not found in catalog");
        return;
    };

    commands.entity(e).insert((transform,));
    spawn_billboard(&mut commands, visual, e);
}

pub fn despawn_order_trigger(
    remove: On<Remove, FlowField>,
    mut commands: Commands,
    triggered: Query<&DelayedStopTrigger>,
    query: Query<(Entity, &Position, &Moving)>,
) {
    query
        .iter()
        .filter(|(_, _, moving)| moving.field == remove.entity)
        .for_each(|(entity, position, _)| {
            commands.entity(entity).remove::<Moving>();
            commands.entity(entity).remove::<Attack>();
            if triggered.contains(entity) {
                commands.entity(entity).remove::<DelayedStopTrigger>();
            }
            commands.entity(entity).insert(Stopped {
                stop_position: **position,
                in_range: true,
                pos_renew_delay: 0.0,
                last_field: Some(remove.entity),
            });
        });
}
