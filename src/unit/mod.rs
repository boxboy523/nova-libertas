pub mod component;
pub mod event;
pub mod spatial_grid;
pub mod system;

use bevy::prelude::*;

//use crate::visual::system::sprite_catalog_startup_system;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(event::spawn_units_trigger)
            .add_observer(event::spawn_wall_trigger)
            .add_systems(
                FixedUpdate,
                (
                    system::despawn_units_system,
                    system::update_spatial_grid_system,
                )
                    .chain(),
            )
            // .add_systems(
            //     Startup,
            //     system::startup_spawn_wall.after(sprite_catalog_startup_system),
            // )
            .add_systems(Update, system::position_to_transform_system);
    }
}
