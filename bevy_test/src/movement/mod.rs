pub mod component;
pub mod event;
pub mod flow_grid;
pub mod move_system;
pub mod nav_system;

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(event::move_order_trigger).add_systems(
            FixedUpdate,
            (
                nav_system::flow_field_added_system,
                nav_system::update_flow_field_system,
                nav_system::flow_movement_system,
                move_system::avoid_system,
                move_system::delayed_stop_system,
                nav_system::remove_empty_orders_system,
                move_system::smooth_wall_passing_system,
                move_system::stop_attacking_unit_system,
                move_system::stop_moving_unit_system,
                move_system::stopped_in_range_system,
                move_system::acceleration_system,
                move_system::apply_move_system,
            )
                .chain(),
        );
    }
}
