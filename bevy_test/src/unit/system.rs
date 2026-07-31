use crate::prelude::*;
use bevy::prelude::*;

pub fn despawn_units_system(mut commands: Commands, dead_query: Query<Entity, With<Dead>>) {
    for entity in dead_query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn update_spatial_grid_system(
    object: Query<(Entity, &Transform, &UnitStats), With<UnitMovement>>,
    mut grid: ResMut<SpatialGrid>,
) {
    grid.clear();
    object.iter().for_each(|(entity, transform, unitstats)| {
        grid.add_entity(entity, transform.translation.xy(), unitstats.size)
            .ok();
    });
}

pub fn startup_spawn_wall(mut commands: Commands, spatial_grid: Res<SpatialGrid>) {
    for x in 0..spatial_grid.width {
        for y in 0..spatial_grid.height {
            if spatial_grid.cells[y * spatial_grid.width + x].is_none() {
                let pos = spatial_grid.grid_to_world(Vec2::new(x as f32, y as f32));
                commands.trigger(SpawnWallEvent { position: pos });
            }
        }
    }
}
