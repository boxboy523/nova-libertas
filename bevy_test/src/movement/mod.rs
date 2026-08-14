pub mod component;
pub mod event;
pub mod flow_grid;
pub mod move_system;
pub mod nav_system;

use bevy::prelude::*;

use crate::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(event::move_order_trigger).add_systems(
            FixedUpdate,
            (
                nav_system::flow_field_added_system,
                nav_system::update_flow_field_system,
                nav_system::flow_movement_system,
                move_system::update_avoid_resp_system,
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

pub fn set_moving(commands: &mut Commands, entity: Entity, moving: Moving) {
    commands.entity(entity).insert(moving);
    commands.entity(entity).remove::<Stopped>();
    commands.entity(entity).remove::<DelayedStopTrigger>();
}

pub fn set_stopped(commands: &mut Commands, entity: Entity, stopped: Stopped) {
    commands.entity(entity).insert(stopped);
    commands.entity(entity).remove::<Moving>();
    commands.entity(entity).remove::<DelayedStopTrigger>();
    commands.entity(entity).remove::<Attack>();
}
