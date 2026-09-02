use crate::prelude::*;
use bevy::prelude::*;

mod components;
mod events;
mod systems;
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (systems::spawn_selection_box, systems::spawn_test_panel).chain(),
        )
        .add_systems(
            Update,
            (
                systems::selection_system,
                systems::update_selection_box,
                systems::move_selected_units,
                systems::spawn_hp_bar,
                systems::update_hp_bar_position_system,
                systems::update_hp_bar_fill_system,
            )
                .chain()
                .after(mouse_input),
        )
        .add_observer(events::remove_hp_bar);
    }
}
