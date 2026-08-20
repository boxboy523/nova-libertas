pub mod component;
pub mod event;
pub mod system;

use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(event::attack_order_trigger)
            .add_observer(event::damage_trigger)
            .add_observer(event::impact_trigger)
            .add_systems(
                FixedUpdate,
                (
                    system::move_or_attack_system,
                    system::auto_attack_system,
                    system::attack_system,
                    system::projectile_system,
                )
                    .chain(),
            );
    }
}
