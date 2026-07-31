pub mod component;
pub mod event;
pub mod spatial_grid;
pub mod system;

use bevy::prelude::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(event::spawn_units_trigger)
            .add_observer(event::spawn_wall_trigger)
            .add_observer(event::despawn_order_trigger)
            .add_systems(
                FixedUpdate,
                (
                    system::despawn_units_system,
                    system::update_spatial_grid_system,
                )
                    .chain(),
            )
            .add_systems(Startup, system::startup_spawn_wall);
    }
}
