use bevy::prelude::*;

use crate::thing::ThingType;

#[derive(Resource, Debug, Default)]
pub struct Resources {
    pub ore: u32,
    pub oil: u32,
    pub human: u32,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerState {
    pub unit_loadout: [Option<ThingType>; 9],
    pub resources: Resources,
}

pub fn player_state_init(mut state: ResMut<PlayerState>) {
    // Initialize the player's unit loadout with default values
    state.unit_loadout = [
        Some(ThingType::AttackerGun),
        Some(ThingType::AttackerCannon),
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
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>()
            .add_systems(Startup, player_state_init);
    }
}
