use bevy::prelude::*;
use bevy_hui::prelude::UiId;

use crate::ui::{
    BottomPanel, UiEntityIndex,
    components::{HpBarRef, TweenBehavior, UiTween},
};

const BOTTOM_PANEL_Z_INDEX: i32 = 200;
const OPENING_BOTTOM_PANEL_Z_INDEX: i32 = BOTTOM_PANEL_Z_INDEX + 1;

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
                    Val::Vw(-26.7),
                    Val::Vh(0.0),
                )),
            );
        }
        "build_panel" | "unit_panel" | "research_panel" => {
            commands.entity(trigger.entity).insert(
                (UiTransform::from_translation(Val2::new(
                    Val::Vw(0.0),
                    Val::Vh(50.0),
                )), GlobalZIndex(BOTTOM_PANEL_Z_INDEX))
            );
        }
        "detail_panel" => {
            commands.entity(trigger.entity).insert(
                (
                    UiTransform::from_translation(Val2::new(
                        Val::Vw(0.0),
                        Val::Vh(40.0),
                    )),
                    GlobalZIndex(210),
                ),
            );
        }
        "build_icon_viewport" | "unit_icon_viewport" | "research_icon_viewport" => {
            commands
                .entity(trigger.entity)
                .insert(ScrollPosition(Vec2::ZERO));
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
                    target_left: TweenBehavior::from_diff(event.0, -22.0, Some(Val::Vw(1.0))),
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
                    target_left: TweenBehavior::from_diff(event.0, 22.0, Some(Val::Vw(1.0))),
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
    if ui_index.entities.get("side_panel") == Some(&event.0) {
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

    for panel in BottomPanel::ALL {
        if ui_index.entities.get(panel.ui_id()) == Some(&event.0) {
            match panel.state(&ui_state) {
                crate::ui::PanelState::Opening => {
                    *panel.state_mut(&mut ui_state) = crate::ui::PanelState::Open;
                }
                crate::ui::PanelState::Closing => {
                    *panel.state_mut(&mut ui_state) = crate::ui::PanelState::Closed;
                }
                _ => panic!(
                    "Tween finished for {:?}, but it was not opening or closing",
                    panel
                ),
            }
            return;
        }
    }

    if ui_index.entities.get("detail_panel") == Some(&event.0) {
        match ui_state.detail_panel {
            crate::ui::PanelState::Opening => {
                ui_state.detail_panel = crate::ui::PanelState::Open;
            }
            crate::ui::PanelState::Closing => {
                ui_state.detail_panel = crate::ui::PanelState::Closed;
            }
            _ => panic!("Tween finished for detail panel, but it was not opening or closing"),
        }
    }
}

#[derive(Event)]
pub struct ToggleBottomPanelEvent(pub BottomPanel);

#[derive(Event)]
pub struct SetDetailPanelOpenEvent(pub bool);

pub fn set_detail_panel_open(
    event: On<SetDetailPanelOpenEvent>,
    mut ui_state: ResMut<crate::ui::UiState>,
    ui_index: Res<crate::ui::UiEntityIndex>,
    transforms: Query<&UiTransform>,
    mut commands: Commands,
) {
    let Some(entity) = ui_index.entities.get("detail_panel") else {
        warn!("Detail panel entity not found in UiEntityIndex");
        return;
    };

    let Ok(transform) = transforms.get(*entity) else {
        warn!("Detail panel entity does not have a UiTransform");
        return;
    };

    let target_up = match (event.0, &ui_state.detail_panel) {
        (true, crate::ui::PanelState::Closed) => {
            ui_state.detail_panel = crate::ui::PanelState::Opening;
            TweenBehavior::from_diff(
                transform.translation.y,
                -40.0,
                Some(Val::Vh(1.0)),
            )
        }
        (false, crate::ui::PanelState::Open) => {
            ui_state.detail_panel = crate::ui::PanelState::Closing;
            TweenBehavior::from_diff(
                transform.translation.y,
                40.0,
                Some(Val::Vh(1.0)),
            )
        }
        _ => return,
    };

    commands.entity(*entity).insert(UiTween {
        state: crate::ui::components::UiTweenState::Moving,
        timer: Timer::from_seconds(0.2, TimerMode::Once),
        target_left: TweenBehavior::ZERO,
        target_up,
    });
}

pub fn toggle_bottom_panel(
    event: On<ToggleBottomPanelEvent>,
    mut ui_state: ResMut<crate::ui::UiState>,
    ui_index: Res<crate::ui::UiEntityIndex>,
    transforms: Query<&UiTransform>,
    mut commands: Commands,
) {
    if BottomPanel::ALL.iter().any(|panel| {
        matches!(
            panel.state(&ui_state),
            crate::ui::PanelState::Opening | crate::ui::PanelState::Closing
        )
    }) {
        return;
    }

    let panel = event.0;
    let Some(&entity) = ui_index.entities.get(panel.ui_id()) else {
        warn!("{} entity not found in UiEntityIndex", panel.ui_id());
        return;
    };
    let Ok(transform) = transforms.get(entity) else {
        warn!("{} does not have a UiTransform", panel.ui_id());
        return;
    };

    let target_up = match panel.state(&ui_state) {
        crate::ui::PanelState::Closed => {
            for open_panel in BottomPanel::ALL {
                if open_panel != panel
                    && open_panel.state(&ui_state) == &crate::ui::PanelState::Open
                {
                    let Some(&open_entity) = ui_index.entities.get(open_panel.ui_id()) else {
                        warn!("{} entity not found in UiEntityIndex", open_panel.ui_id());
                        return;
                    };
                    let Ok(open_transform) = transforms.get(open_entity) else {
                        warn!("{} does not have a UiTransform", open_panel.ui_id());
                        return;
                    };

                    *open_panel.state_mut(&mut ui_state) = crate::ui::PanelState::Closing;
                    commands.entity(open_entity).insert((
                        GlobalZIndex(BOTTOM_PANEL_Z_INDEX),
                        UiTween {
                            state: crate::ui::components::UiTweenState::Moving,
                            timer: Timer::from_seconds(0.2, TimerMode::Once),
                            target_left: TweenBehavior::ZERO,
                            target_up: TweenBehavior::from_diff(
                                open_transform.translation.y,
                                50.0,
                                Some(Val::Vh(1.0)),
                            ),
                        },
                    ));
                }
            }

            *panel.state_mut(&mut ui_state) = crate::ui::PanelState::Opening;
            commands
                .entity(entity)
                .insert(GlobalZIndex(OPENING_BOTTOM_PANEL_Z_INDEX));
            TweenBehavior::from_diff(
                transform.translation.y,
                -50.0,
                Some(Val::Vh(1.0)),
            )
        }
        crate::ui::PanelState::Open => {
            *panel.state_mut(&mut ui_state) = crate::ui::PanelState::Closing;
            commands
                .entity(entity)
                .insert(GlobalZIndex(BOTTOM_PANEL_Z_INDEX));
            TweenBehavior::from_diff(
                transform.translation.y,
                50.0,
                Some(Val::Vh(1.0)),
            )
        }
        _ => return,
    };

    let detail_panel_open = matches!(panel.state(&ui_state), crate::ui::PanelState::Opening);

    commands.entity(entity).insert(UiTween {
        state: crate::ui::components::UiTweenState::Moving,
        timer: Timer::from_seconds(0.2, TimerMode::Once),
        target_left: TweenBehavior::ZERO,
        target_up,
    });
    commands.trigger(SetDetailPanelOpenEvent(detail_panel_open));
}
