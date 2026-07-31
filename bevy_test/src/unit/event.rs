use crate::prelude::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnUnitEvent {
    pub transform: Transform,
    pub t_type: ThingType,
    pub team: Team,
    pub hp: f32, // 유닛의 초기 체력
}

pub fn spawn_units_trigger(
    event: On<SpawnUnitEvent>,
    mut commands: Commands,
    catalog: Res<ThingCatalog>,
    asset_server: Res<AssetServer>,
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
    let mut sprite = Sprite::from_image(asset_server.load(info.sprite_info.img_path.clone()));
    sprite.custom_size = info.sprite_info.size.into();
    commands.spawn((
        sprite,
        event.transform,
        info.unit_stats.unwrap(),
        info.battle_stats.unwrap(),
        event.team,
        Stopped {
            stop_position: event.transform.translation.xy(),
            in_range: true,
            ..Default::default()
        },
        UnitMovement::default(),
        UnitHp(event.hp),
        event.t_type,
    ));
}

#[derive(Event)]
pub struct SpawnWallEvent {
    pub position: Vec2,
}

pub fn spawn_wall_trigger(
    event: On<SpawnWallEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<ThingCatalog>,
) {
    let e = commands.spawn_empty().id();
    let transform = Transform {
        translation: event.position.extend(0.0),
        ..Default::default()
    };
    let Some(info) = catalog.get_info(ThingType::Wall) else {
        warn!("ThingType Wall not found in catalog");
        return;
    };
    let mut sprite = Sprite::from_image(asset_server.load(info.sprite_info.img_path.clone()));
    sprite.custom_size = info.sprite_info.size.into();
    commands.entity(e).insert((transform, sprite));
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
                stop_position: transform.translation.xy(),
                in_range: true,
                pos_renew_delay: 0.0,
                last_field: Some(remove.entity),
            });
        });
}
