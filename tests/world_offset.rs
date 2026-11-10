use std::time::Duration;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use nova_libertas::{
    combat::{
        component::{AttackImpact, Projectile, ProjectilePosition},
        system::projectile_system,
    },
    map::TerrainHeightMap,
    unit::{
        component::{Position, Team, UnitStats},
        event::{SpawnWallEvent, spawn_wall_trigger},
        system::position_to_transform_system,
    },
    visual::{SpriteCatalog, UnitVisual, UnitVisualKind},
    world3d::{Billboard, spawn_billboard},
    thing::ThingType,
};

fn visual(world_offset: Vec3) -> UnitVisual {
    UnitVisual {
        kind: UnitVisualKind::Simple {
            mesh: Handle::default(),
            material: [(Team::Neutral, Handle::default())].into_iter().collect(),
        },
        size: Vec2::splat(20.0),
        world_offset,
        anchor: None,
        roll_offset: 0.0,
    }
}

fn attach_visual(world: &mut World, entity: Entity, world_offset: Vec3) {
    world.run_system_once(move |mut commands: Commands| {
        spawn_billboard(&mut commands, &visual(world_offset), entity, None);
    }).unwrap();
}

#[test]
fn unit_world_offset_is_world_aligned_and_does_not_accumulate() {
    let mut world = World::new();
    world.insert_resource(TerrainHeightMap {
        width: 4,
        height: 4,
        sample_spacing: 20.0,
        heights: vec![7.0; 16],
        mesh: None,
    });
    let camera = world.spawn((Camera3d::default(), GlobalTransform::from(
        Transform::from_rotation(Quat::from_rotation_x(-0.7)),
    ))).id();
    let position = Vec2::new(20.0, 30.0);
    let offset = Vec3::new(3.0, 12.0, -5.0);
    let entities = [Vec3::ZERO, offset].map(|offset| {
        let entity = world.spawn((
            Position(position),
            UnitStats { size: 5.0, max_speed: 10.0, acceleration: 10.0 },
            Transform::default(),
        )).id();
        attach_visual(&mut world, entity, offset);
        entity
    });
    for rotation in [Quat::from_rotation_x(-0.7), Quat::from_rotation_y(0.8) * Quat::from_rotation_x(-0.4)] {
        world.entity_mut(camera).insert(GlobalTransform::from(Transform::from_rotation(rotation)));
        for _ in 0..2 {
            world.run_system_cached(position_to_transform_system).unwrap();
            let original = world.get::<Transform>(entities[0]).unwrap().translation;
            let shifted = world.get::<Transform>(entities[1]).unwrap().translation;
            assert!((shifted - original - offset).length() < 0.001);
            for entity in entities {
                assert_eq!(world.get::<Position>(entity).unwrap().0, position);
            }
        }
    }
}

#[test]
fn projectile_render_offset_does_not_change_flight_roll_or_arrival() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs_f32(0.5));
    world.insert_resource(time);
    world.spawn((Camera3d::default(), GlobalTransform::default()));
    let target = world.spawn(Position(Vec2::new(10.0, 0.0))).id();
    let offset = Vec3::new(3.0, 12.0, -5.0);
    let projectiles = [Vec3::ZERO, offset].map(|offset| {
        let entity = world.spawn((
            Projectile {
                sender: target,
                target,
                damage: 1.0,
                speed: 4.0,
                impact: AttackImpact::Single,
                team: Team::Neutral,
            },
            ProjectilePosition(Vec3::ZERO),
            Transform::from_translation(offset),
        )).id();
        attach_visual(&mut world, entity, offset);
        entity
    });
    for tick in 1..=4 {
        world.run_system_cached(projectile_system).unwrap();
        for (entity, offset) in projectiles.into_iter().zip([Vec3::ZERO, offset]) {
            let logical_position = world.get::<ProjectilePosition>(entity).unwrap().0;
            assert_eq!(logical_position, Vec3::new(tick as f32 * 2.0, 0.0, 0.0));
            assert_eq!(world.get::<Transform>(entity).unwrap().translation, logical_position + offset);
        }
        assert_eq!(world.get::<Billboard>(projectiles[0]).unwrap().roll, world.get::<Billboard>(projectiles[1]).unwrap().roll);
    }
    world.run_system_cached(projectile_system).unwrap();
    for entity in projectiles {
        assert!(world.get::<Projectile>(entity).is_none());
    }
}

#[test]
fn static_wall_applies_world_offset_once_at_spawn() {
    let mut world = World::new();
    let offset = Vec3::new(3.0, 12.0, -5.0);
    let mut catalog = SpriteCatalog::default();
    catalog.sprites.insert(ThingType::Wall, visual(offset));
    world.insert_resource(catalog);
    world.add_observer(spawn_wall_trigger);
    let position = Vec2::new(20.0, 30.0);
    world.trigger(SpawnWallEvent { position });
    world.flush();
    let mut query = world.query::<(&Transform, &Billboard)>();
    let (transform, billboard) = query.single(&world).unwrap();
    assert_eq!(transform.translation, Vec3::new(20.0, 0.0, 30.0) + offset);
    assert_eq!(billboard.world_offset, offset);
}
