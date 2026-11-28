use bevy::prelude::*;

use crate::thing::ThingId;

#[derive(Resource, Debug, Default)]
pub struct Resources {
    pub ore: u32,
    pub oil: u32,
    pub human: u32,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerState {
    pub unit_loadout: [Option<ThingId>; 9],
    pub building_loadout: BuildingLoadout,
    pub resources: Resources,
}

#[derive(Debug, Default)]
pub struct BuildingLoadout {
    pub core: [Option<ThingId>; 4],
    pub production: [Option<ThingId>; 4],
    pub research: [Option<ThingId>; 4],
    pub defense: [Option<ThingId>; 4],
}

pub fn player_state_init(mut state: ResMut<PlayerState>) {
    // Initialize the player's unit loadout with default values
    state.unit_loadout = [
        Some(ThingId::from_name("unit:attackerGun")),
        Some(ThingId::from_name("unit:attackerCannon")),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];
    state.resources = Resources {
        ore: 100,
        oil: 50,
        human: 10,
    };
    state.building_loadout = BuildingLoadout {
        core: [
            Some(ThingId::from_name("building:castle")),
            None,
            None,
            None,
        ],
        production: [
            Some(ThingId::from_name("building:buildAcademy")),
            None,
            None,
            None,
        ],
        research: [
            Some(ThingId::from_name("building:laboratory")),
            None,
            None,
            None,
        ],
        defense: [
            Some(ThingId::from_name("building:transmitter")),
            None,
            None,
            None,
        ],
    };
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>()
            .add_systems(Startup, player_state_init);
    }
}
