use bevy::prelude::*;
use bevy_hui::prelude::UiId;

use crate::ui::{UiEntityIndex, components::{HpBarRef, TweenBehavior, UiTween}};

pub fn remove_hp_bar(
    trigger: On<Remove, HpBarRef>,
    query: Query<&HpBarRef>,
    mut commands: Commands,
) {
    if let Ok(hp_bar) = query.get(trigger.entity) {
        commands.entity(hp_bar.root).despawn();
    } else {
        warn!(
            "Failed to find HpBarRef for entity {:?} when trying to remove hp bar",
            trigger.entity
        );
    }
}

pub fn add_uiid(
    trigger: On<Add, UiId>,
    query: Query<&UiId>,
    mut ui_index: ResMut<UiEntityIndex>,
    mut commands: Commands,
) {
    let Ok(ui_id) = query.get(trigger.entity) else {
        warn!("Failed to read added UiId");
        return;
    };
    ui_index
        .entities
        .insert(ui_id.as_str().to_owned(), trigger.entity);

    match ui_id.as_str() {
        "side_panel" => {
            commands.entity(trigger.entity).insert(
                UiTransform::from_translation(Val2::new(
                    Val::Vw(-22.7),
                    Val::Vh(0.0),
                )),
            );
        }

        _ => {}
    }
}

pub fn remove_uiid(
    trigger: On<Remove, UiId>,
    query: Query<&UiId>,
    mut ui_index: ResMut<crate::ui::UiEntityIndex>,
) {
    if let Ok(ui_id) = query.get(trigger.entity) {
        if ui_index.entities.get(ui_id.as_str()) == Some(&trigger.entity) {
            ui_index.entities.remove(ui_id.as_str());
        } else {
            warn!(
                "UiId {:?} for entity {:?} does not match the entity in the index",
                ui_id, trigger.entity
            );
        }
    } else {
        warn!(
            "Failed to find UiId for entity {:?} when trying to remove from index",
            trigger.entity
        );
    }
}

#[derive(Event)]
pub struct ToggleSidePanelEvent(pub Val);
pub fn toggle_side_panel(
    event: On<ToggleSidePanelEvent>,
    mut ui_state: ResMut<crate::ui::UiState>,
    ui_index: Res<crate::ui::UiEntityIndex>,
    mut commands: Commands,
) {
    match ui_state.side_panel {
        crate::ui::PanelState::Open => {
            ui_state.side_panel = crate::ui::PanelState::Closing;
            if let Some(entity) = ui_index.entities.get("side_panel") {
                commands.entity(*entity).insert(UiTween {
                    state: crate::ui::components::UiTweenState::Moving,
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                    target_left: TweenBehavior::from_diff(event.0, -22.0),
                    target_up: TweenBehavior::ZERO,
                });
            }
            println!("Closing side panel");
        }
        crate::ui::PanelState::Closed => {
            ui_state.side_panel = crate::ui::PanelState::Opening;
            if let Some(entity) = ui_index.entities.get("side_panel") {
                commands.entity(*entity).insert(UiTween {
                    state: crate::ui::components::UiTweenState::Moving,
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                    target_left: TweenBehavior::from_diff(event.0, 22.0),
                    target_up: TweenBehavior::ZERO,
                });
            }
            println!("Opening side panel");
        }
        _ => {}
    }
}

#[derive(Event)]
pub struct TweenFinishedEvent(pub Entity);
pub fn tween_finished(
    event: On<TweenFinishedEvent>,
    mut ui_state: ResMut<crate::ui::UiState>,
    ui_index: Res<crate::ui::UiEntityIndex>,
) {
    if let Some(entity) = ui_index.entities.get("side_panel") {
        if *entity == event.0 {
            match ui_state.side_panel {
                crate::ui::PanelState::Opening => {
                    ui_state.side_panel = crate::ui::PanelState::Open;
                }
                crate::ui::PanelState::Closing => {
                    ui_state.side_panel = crate::ui::PanelState::Closed;
                }
                _ => panic!("Tween finished for side panel, but it was not opening or closing"),
            }
        }
    }
}
