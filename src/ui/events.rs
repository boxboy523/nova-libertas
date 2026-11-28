use bevy::prelude::*;
use bevy_hui::prelude::UiId;

use crate::{player::PlayerState, thing::{ThingCatalog, ThingId}, ui::{
    BottomPanel, UiEntityIndex,
    components::{HpBarRef, TweenBehavior, UiTween, UnitSlot},
}, unit::component::{Selected, Team}};

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
    player_state: Res<PlayerState>,
    thing_catalog: Res<ThingCatalog>,
    ui_state: Res<crate::ui::UiState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok(ui_id) = query.get(trigger.entity) else {
        warn!("Failed to read added UiId");
        return;
    };
    ui_index
        .entities
        .insert(ui_id.as_str().to_owned(), trigger.entity);

    if let Some(slot) = ui_id
        .as_str()
        .strip_prefix("unit_")
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| (1..=9).contains(&n)) {
        let index = slot - 1;
        if let Some(unit_id) = player_state.unit_loadout[index] {
            commands.entity(trigger.entity).insert(UnitSlot(index)).observe(on_unit_slot_clicked);
            if let Some(info) = thing_catalog.get_info(unit_id) {
                if let Some(path) = &info.icon {
                    let icon = commands.spawn((
                        ImageNode::new(asset_server.load(path.clone())),
                        Node { height: Val::Percent(75.0), aspect_ratio: Some(1.0), ..default() },
                        Pickable::IGNORE,
                    )).id();
                    commands.entity(trigger.entity).add_child(icon);
                }
            } else {
                warn!("Unknown unit in loadout slot {slot}: {unit_id:?}");
            }
        }
    }
    let name = ui_id.as_str();
    if name.starts_with("building:") {
        let building_id = ThingId::from_name(name);
        if thing_catalog.get_info(building_id).and_then(|info| info.building_stats.as_ref()).is_none() {
            warn!("Unknown or non-buildable thing for build button: {name}");
            return;
        }
        commands.entity(trigger.entity).observe(
            move |mut click: On<Pointer<Click>>, mut commands: Commands| {
                if click.button == bevy::picking::pointer::PointerButton::Primary {
                    click.propagate(false);
                    commands.trigger(crate::building::BeginBuildingPlacement(building_id));
                }
            },
        );
        return;
    }
    match name {
        "options_panel" => {
            commands.entity(trigger.entity).insert((
                GlobalZIndex(300),
                if ui_state.options_open {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ));
        }
        "options_button" | "options_panel_close" => {
            let open = ui_id.as_str() == "options_button";
            commands.entity(trigger.entity).observe(
                move |mut click: On<Pointer<Click>>, mut commands: Commands| {
                    if click.button == bevy::picking::pointer::PointerButton::Primary {
                        click.propagate(false);
                        commands.trigger(SetOptionsPanelOpenEvent(open));
                    }
                },
            );
        }
        "detail_panel_close" => {
            commands.entity(trigger.entity).observe(
                |mut click: On<Pointer<Click>>,
                 ui_state: Res<crate::ui::UiState>,
                 mut commands: Commands| {
                    if click.button != bevy::picking::pointer::PointerButton::Primary {
                        return;
                    }
                    click.propagate(false);
                    if let Some(panel) = BottomPanel::ALL.into_iter().find(|panel| {
                        panel.state(&ui_state) == &crate::ui::PanelState::Open
                    }) {
                        commands.trigger(ToggleBottomPanelEvent(panel));
                    }
                },
            );
        }
        "unit_all" => {
            commands.entity(trigger.entity).observe(
                |click: On<Pointer<Click>>, mut commands: Commands| {
                    if click.button == bevy::picking::pointer::PointerButton::Primary {
                        commands.trigger(SelectAllUnitsEvent);
                    }
                },
            );
        }
        "red_panel" | "blue_panel" | "green_panel" => {
            let panel = match ui_id.as_str() {
                "red_panel" => BottomPanel::Build,
                "blue_panel" => BottomPanel::Unit,
                "green_panel" => BottomPanel::Research,
                _ => unreachable!(),
            };
            commands.entity(trigger.entity).observe(
                move |click: On<Pointer<Click>>, mut commands: Commands| {
                    if click.button == bevy::picking::pointer::PointerButton::Primary {
                        commands.trigger(ToggleBottomPanelEvent(panel));
                    }
                },
            );
        }
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
        "unit_icon_grid" => {
            spawn_unit_grid(&mut commands, trigger.entity, &player_state, &thing_catalog, &asset_server);
        }
        "build_icon_grid" => {
            spawn_building_grid(&mut commands, trigger.entity, &player_state, &thing_catalog, &asset_server);
        }
        _ => {}
    }
}

fn spawn_unit_grid(
    commands: &mut Commands,
    parent: Entity,
    player_state: &PlayerState,
    thing_catalog: &ThingCatalog,
    asset_server: &AssetServer,
) {
    let mut tier_info = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    player_state.unit_loadout.iter().for_each(|&unit_id| {
        let Some(unit_id) = unit_id else {
            return;
        };
        let Some(info) = thing_catalog.get_info(unit_id) else {
            warn!("Unknown unit in loadout: {unit_id:?}");
            return;
        };
        let Some(prod) = info.unit_production.as_ref() else {
            warn!("ThingType {:?} does not have unit_production", unit_id);
            return;
        };
        tier_info[(prod.tier - 1) as usize].push(info.clone());
    });
    let max_length = tier_info.iter().map(|v| v.len()).max().unwrap_or(0);
    tier_info.iter().for_each(|infos| {
        infos.iter().for_each(|info| {
            let Some(path) = &info.icon else {
                warn!("ThingType {:?} does not have an icon", info);
                return;
            };
            let slot = commands.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ChildOf(parent),
            )).observe(move |mut click: On<Pointer<Click>>, mut commands: Commands| {
                if click.button == bevy::picking::pointer::PointerButton::Primary {
                    click.propagate(false);
                    //TODO: Trigge spawn unit event for this unit type
                }
            }).id();
            commands.spawn((
                ImageNode::new(asset_server.load(path.clone())),
                Node { height: Val::Percent(90.0), aspect_ratio: Some(1.0), ..default() },
                ChildOf(slot),
                Pickable::IGNORE,
            ));
        });
        (infos.len()..max_length).for_each(|_| {
            commands.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ChildOf(parent),
            ));
        });
    });
}

fn spawn_building_grid(
    commands: &mut Commands,
    parent: Entity,
    player_state: &PlayerState,
    thing_catalog: &ThingCatalog,
    asset_server: &AssetServer,
) {
    let mut spawn_slot = |&building_id| {
        let mut slot = commands.spawn((
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ChildOf(parent),
        ));
        let Some(id) = building_id else {
            return;
        };
        let Some(info) = thing_catalog.get_info(id) else {
            warn!("Unknown building in loadout: {building_id:?}");
            return;
        };
        let Some(path) = &info.icon else {
            warn!("ThingType {:?} does not have an icon", info);
            return;
        };
        let slot = slot.observe(move |mut click: On<Pointer<Click>>, mut commands: Commands| {
            if click.button == bevy::picking::pointer::PointerButton::Primary {
                click.propagate(false);
                commands.trigger(crate::building::BeginBuildingPlacement(id));
            }
        }).id();
        commands.spawn((
            ImageNode::new(asset_server.load(path.clone())),
            Node { height: Val::Percent(90.0), aspect_ratio: Some(1.0), ..default() },
            ChildOf(slot),
            Pickable::IGNORE,
        ));
    };

    (0..4).for_each(|i| {
        spawn_slot(&player_state.building_loadout.core[i]);
        spawn_slot(&player_state.building_loadout.production[i]);
        spawn_slot(&player_state.building_loadout.research[i]);
        spawn_slot(&player_state.building_loadout.defense[i]);
    });
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
pub struct SetOptionsPanelOpenEvent(pub bool);

pub fn set_options_panel_open(
    event: On<SetOptionsPanelOpenEvent>,
    mut ui_state: ResMut<crate::ui::UiState>,
    ui_index: Res<UiEntityIndex>,
    mut visibility: Query<&mut Visibility>,
) {
    let Some(&entity) = ui_index.entities.get("options_panel") else {
        return;
    };
    let Ok(mut visibility) = visibility.get_mut(entity) else {
        return;
    };
    ui_state.options_open = event.0;
    *visibility = if event.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

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

pub fn on_unit_slot_clicked(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    unit_slot_query: Query<&UnitSlot>,
) {
    if trigger.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }
    let Ok(unit_slot) = unit_slot_query.get(trigger.entity) else {
        warn!("Clicked entity does not have a UnitSlot component");
        return;
    };
    commands.trigger(SelectUnitSlotEvent { index: unit_slot.0 });
}

/// Select all player units of the type assigned to a zero-based loadout slot.
#[derive(Event)]
pub struct SelectUnitSlotEvent {
    pub index: usize,
}

pub fn select_unit_slot(
    event: On<SelectUnitSlotEvent>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    units: Query<(Entity, &ThingId, &Team)>,
    player_state: Res<PlayerState>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    if let Some(&Some(unit_id)) = player_state.unit_loadout.get(event.index) {
        ui_state.last_selected_slot = Some(event.index);
        selected_query.iter().for_each(|entity| {
            commands.entity(entity).remove::<Selected>();
        });
        units.iter().for_each(|(entity, id, team)| {
            if *id == unit_id && *team == Team::Player {
                commands.entity(entity).insert(Selected);
            }
        });
    }
}

#[derive(Event)]
pub struct SelectAllUnitsEvent;

pub fn select_all_units(
    _event: On<SelectAllUnitsEvent>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    teams: Query<(Entity, &Team), With<crate::movement::component::UnitMovement>>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    ui_state.last_selected_slot = None;
    for entity in &selected_query {
        commands.entity(entity).remove::<Selected>();
    }
    for (entity, team) in &teams {
        if *team == Team::Player {
            commands.entity(entity).insert(Selected);
        }
    }
}
