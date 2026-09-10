use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

mod components;
pub mod events;
mod systems;
pub struct UIPlugin;


#[derive(Resource, Default)]
pub struct UiEntityIndex {
    pub entities: HashMap<String, Entity>,
}

#[derive(Resource, Default)]
pub struct UiState {
    pub side_panel: PanelState,
}

#[derive(Default)]
pub enum PanelState {
    Open,
    Opening,
    #[default]
    Closed,
    Closing,
}


impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiEntityIndex>()
            .init_resource::<UiState>()
            .add_systems(
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
            .add_systems(
                Update,
                    systems::animate_ui_tween,
            )
            .add_observer(events::remove_hp_bar).add_observer(events::add_uiid)
            .add_observer(events::remove_uiid)
            .add_observer(events::toggle_side_panel)
            .add_observer(events::tween_finished);
    }
}
