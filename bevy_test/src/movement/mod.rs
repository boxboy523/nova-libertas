pub mod component;
pub mod event;
pub mod flow_grid;
pub mod system;

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(event::move_order_trigger).add_systems(
            FixedUpdate,
            (
                system::flow_field_added_system,
                system::update_flow_field_system,
                system::flow_movement_system,
                system::avoid_system,
                system::delayed_stop_system,
                system::remove_empty_orders_system,
                system::smooth_wall_passing_system,
                system::stop_attacking_unit_system,
                system::stop_moving_unit_system,
                system::stopped_in_range_system,
                system::acceleration_system,
                system::apply_move_system,
            )
                .chain(),
        );
    }
}
