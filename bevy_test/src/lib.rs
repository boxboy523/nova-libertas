pub mod combat;
pub mod constants;
pub mod debug;
pub mod input;
pub mod movement;
pub mod thing;
pub mod ui;
pub mod unit;

pub mod prelude {
    pub use crate::combat::{
        component::{Attack, AutoAttack, UnitBattleStats, UnitHp},
        event::{AttackOrderEvent, DamageEvent},
        CombatPlugin,
    };
    pub use crate::constants::*;
    pub use crate::debug::DebugPlugin;
    pub use crate::input::{InputPlugin, MouseState};
    pub use crate::movement::{
        component::{
            DelayedStopTrigger, FieldFollowTarget, FlowField, Moving, Stopped, UnitMovement,
        },
        event::MoveOrderEvent,
        flow_grid::FlowGrid,
        MovementPlugin,
    };
    pub use crate::thing::{ThingCatalog, ThingInfo, ThingType};
    pub use crate::unit::{
        component::{Dead, Selected, Team, UnitStats},
        event::{SpawnUnitEvent, SpawnWallEvent},
        spatial_grid::{CollisionResult, SpatialGrid},
        UnitPlugin,
    };
}

pub fn load_map_from_csv(path: &str) -> (usize, usize, Vec<bool>) {
    let content = std::fs::read_to_string(path).unwrap();
    let mut wall = Vec::new();
    let mut width = 0;
    let mut height = 0;
    for line in content.lines() {
        height += 1;
        let row: Vec<bool> = line.split(',').map(|c| c.trim() == "1").collect();
        width = row.len();
        wall.extend(row);
    }
    (width, height, wall)
}

use bevy::prelude::*;
use prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(500.0, 500.0, 0.0)));
    for i in 0..5 {
        for j in 0..5 {
            commands.trigger(SpawnUnitEvent {
                transform: Transform::from_xyz(i as f32 * 40.0 + 40.0, j as f32 * 40.0 + 40.0, 0.0),
                t_type: ThingType::AttackerGun,
                team: Team::Player,
                hp: 100.0,
            });
        }
    }
}
