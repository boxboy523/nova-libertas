use std::time::Duration;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use nova_libertas::{
    combat::{
        component::{AttackImpact, Projectile},
        system::projectile_system,
    },
    map::TerrainHeightMap,
    unit::{
        component::{Position, Team, UnitStats},
        event::{SpawnWallEvent, spawn_wall_trigger},
        system::position_to_transform_system,
    },
    thing::ThingType,
    visual::{SpriteCatalog, UnitVisual, UnitVisualKind, info::VisualAnchor},
    world3d::{Billboard, spawn_billboard},
};

fn visual(model_offset: Vec3) -> UnitVisual {
    UnitVisual {
        kind: UnitVisualKind::Simple {
            mesh: Handle::default(),
            material: [(Team::Neutral, Handle::default())].into_iter().collect(),
        },
        size: Vec2::splat(20.0),
        model_offset,
        anchor: None,
        roll_offset: 0.0,
    }
}

fn attach_visual(world: &mut World, parent: Entity, model_offset: Vec3) -> Entity {
    world
        .run_system_once(move |mut commands: Commands| {
            spawn_billboard(&mut commands, &visual(model_offset), parent, None)
        })
        .unwrap()
}

#[test]
fn unit_model_offset_is_world_aligned_and_does_not_accumulate() {
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
    let ground_y = world.resource::<TerrainHeightMap>().height_at(position);
    let offset = Vec3::new(3.0, 12.0, -5.0);
    let entities = [Vec3::ZERO, offset].map(|offset| {
        let entity = world.spawn((
            Position(position),
            UnitStats { size: 5.0, max_speed: 10.0, acceleration: 10.0 },
            Transform::default(),
        )).id();
        let child = attach_visual(&mut world, entity, offset);
        (entity, child)
    });
    for rotation in [Quat::from_rotation_x(-0.7), Quat::from_rotation_y(0.8) * Quat::from_rotation_x(-0.4)] {
        world.entity_mut(camera).insert(GlobalTransform::from(Transform::from_rotation(rotation)));
        for _ in 0..2 {
            world.run_system_cached(position_to_transform_system).unwrap();
            let [(base_parent, base_child), (shifted_parent, shifted_child)] = entities;
            let base = world.get::<Transform>(base_parent).unwrap();
            let shifted = world.get::<Transform>(shifted_parent).unwrap();
            assert_eq!(base.translation, shifted.translation);
            assert_eq!(base.translation.y, ground_y);

            let base_local = world.get::<Transform>(base_child).unwrap().translation;
            let shifted_local = world.get::<Transform>(shifted_child).unwrap().translation;
            let actual_offset = shifted.transform_point(shifted_local) - base.transform_point(base_local);
            assert!((actual_offset - offset).length() < 0.001);
            for (parent, child) in entities {
                assert_eq!(world.get::<Position>(parent).unwrap().0, position);
                assert_eq!(world.get::<ChildOf>(child).unwrap().parent(), parent);
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
    let target = world
        .spawn((
            Position(Vec2::new(6.0, 0.0)),
            Transform::from_xyz(99.0, 5.0, 99.0),
            VisualAnchor {
                hit: Vec2::new(0.0, 10.0),
                ..default()
            },
        ))
        .id();
    let start = Vec3::new(0.0, 7.0, 0.0);
    let direction = Vec3::new(0.6, 0.8, 0.0);
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
            Transform::from_translation(start),
        )).id();
        let child = attach_visual(&mut world, entity, offset);
        (entity, child)
    });
    for tick in 1..=4 {
        world.run_system_cached(projectile_system).unwrap();
        let expected_position = start + direction * (tick as f32 * 2.0);
        for ((parent, child), offset) in projectiles.into_iter().zip([Vec3::ZERO, offset]) {
            let position = world.get::<Transform>(parent).unwrap().translation;
            assert!((position - expected_position).length() < 0.001);
            assert_eq!(world.get::<Transform>(child).unwrap().translation, offset);
            let roll = world.get::<Billboard>(child).unwrap().roll;
            assert!((roll - direction.y.atan2(direction.x)).abs() < 0.001);
        }
    }
    for _ in 0..2 {
        // Allow one extra fixed tick if rounding puts distance just over the last step.
        world.run_system_cached(projectile_system).unwrap();
        let [first, second] = projectiles.map(|(parent, _)| world.get_entity(parent).is_ok());
        assert_eq!(first, second, "render offset changed the arrival tick");
        if !first {
            break;
        }
    }
    for (parent, child) in projectiles {
        assert!(world.get_entity(parent).is_err());
        assert!(world.get_entity(child).is_err());
    }
}

#[test]
fn static_wall_applies_model_offset_once_at_spawn() {
    let mut world = World::new();
    let offset = Vec3::new(3.0, 12.0, -5.0);
    let mut catalog = SpriteCatalog::default();
    catalog.sprites.insert(ThingType::Wall, visual(offset));
    world.insert_resource(catalog);
    world.add_observer(spawn_wall_trigger);
    let position = Vec2::new(20.0, 30.0);
    world.trigger(SpawnWallEvent { position });
    world.flush();
    let mut query = world.query_filtered::<(&ChildOf, &Transform), With<Billboard>>();
    let (child_of, child_transform) = query.single(&world).unwrap();
    let parent_transform = world.get::<Transform>(child_of.parent()).unwrap();
    let expected_base = Vec3::new(20.0, 0.0, 30.0);
    assert_eq!(parent_transform.translation, expected_base);
    assert_eq!(child_transform.translation, offset);
    assert_eq!(
        parent_transform.transform_point(child_transform.translation),
        expected_base + offset,
    );
}
