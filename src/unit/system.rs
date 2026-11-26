use crate::{
    map::TerrainHeightMap,
    movement::component::UnitMovement,
    unit::{component::{Dead, Position, UnitStats}, spatial_grid::SpatialGrid},
};
use bevy::prelude::*;

pub fn despawn_units_system(mut commands: Commands, dead_query: Query<Entity, With<Dead>>) {
    for entity in dead_query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn update_spatial_grid_system(
    object: Query<(Entity, &Position, &UnitStats), With<UnitMovement>>,
    mut grid: ResMut<SpatialGrid>,
) {
    grid.clear();
    object.iter().for_each(|(entity, position, unitstats)| {
        grid.add_entity(entity, **position, unitstats.size).ok();
    });
}

pub fn position_to_transform_system(
    mut query: Query<(&Position, &UnitStats, &mut Transform)>,
    height_map: Res<TerrainHeightMap>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
) {
    let camera_up = camera.up().as_vec3();
    let horizontal_up = Vec3::new(camera_up.x, 0.0, camera_up.z);
    for (position, stats, mut transform) in query.iter_mut() {
        let height = height_map.height_at(**position);
        let ground_position = Vec3::new(position.x, height, position.y);
        transform.translation = ground_position - horizontal_up * stats.size;
    }
}

// pub fn update_unit_depth_system(mut query: Query<&mut Transform, With<UnitStats>>) {
//     for mut transform in query.iter_mut() {
//         transform.translation.z = 10.0 - transform.translation.y * 0.001;
//     }
// }
