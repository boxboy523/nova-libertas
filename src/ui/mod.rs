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
    pub build_panel: PanelState,
    pub unit_panel: PanelState,
    pub research_panel: PanelState,
    pub detail_panel: PanelState,
}

#[derive(Default, PartialEq, Eq)]
pub enum PanelState {
    Open,
    Opening,
    #[default]
    Closed,
    Closing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BottomPanel {
    Build,
    Unit,
    Research,
}

impl BottomPanel {
    pub const ALL: [Self; 3] = [Self::Build, Self::Unit, Self::Research];

    pub fn ui_id(self) -> &'static str {
        match self {
            Self::Build => "build_panel",
            Self::Unit => "unit_panel",
            Self::Research => "research_panel",
        }
    }

    pub fn state(self, ui_state: &UiState) -> &PanelState {
        match self {
            Self::Build => &ui_state.build_panel,
            Self::Unit => &ui_state.unit_panel,
            Self::Research => &ui_state.research_panel,
        }
    }

    pub fn state_mut(self, ui_state: &mut UiState) -> &mut PanelState {
        match self {
            Self::Build => &mut ui_state.build_panel,
            Self::Unit => &mut ui_state.unit_panel,
            Self::Research => &mut ui_state.research_panel,
        }
    }
}


impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiEntityIndex>()
            .init_resource::<UiState>()
            .add_systems(
            Startup,
                (
                    systems::spawn_selection_box,
                    systems::register_ui_components,
                    systems::spawn_main_ui,
                )
                    .chain(),
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
            .add_observer(events::toggle_bottom_panel)
            .add_observer(events::set_detail_panel_open)
            .add_observer(events::tween_finished);
    }
}
