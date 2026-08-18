pub mod combat;
pub mod constants;
pub mod debug;
pub mod input;
pub mod movement;
pub mod thing;
pub mod ui;
pub mod unit;
pub mod visual;
pub mod world3d;

pub mod prelude {
    pub use crate::combat::{
        component::{Attack, AutoAttack, Projectile, UnitBattleStats, UnitHp},
        event::{AttackOrderEvent, DamageEvent},
        CombatPlugin,
    };
    pub use crate::constants::*;
    pub use crate::debug::DebugPlugin;
    pub use crate::input::{mouse_input, screen_to_ground, InputPlugin, MouseState};
    pub use crate::movement::{
        component::{
            DelayedStopTrigger, FieldFollowTarget, FlowField, Moving, Stopped, UnitMovement,
        },
        event::MoveOrderEvent,
        flow_grid::FlowGrid,
        set_moving, set_stopped, MovementPlugin,
    };
    pub use crate::thing::{ThingCatalog, ThingInfo, ThingType};
    pub use crate::ui::UIPlugin;
    pub use crate::unit::{
        component::{Dead, Position, Selected, Team, UnitStats},
        event::{SpawnUnitEvent, SpawnWallEvent},
        spatial_grid::{CollisionResult, SpatialGrid},
        UnitPlugin,
    };
    pub use crate::visual::{
        data::{
            AnimationData, AnimationFrameMesh, AnimationKind, AnimationSet, AnimationState,
            CurrentAnimation,
        },
        info::{SpriteConfig, SpriteInfo, SpriteInfoKind, VisualAnchor},
        team_color::TeamColorMaterial,
        SpriteCatalog, SpritePlugin, UnitVisual, UnitVisualKind,
    };
    pub use crate::world3d::{create_atlas_quad, spawn_billboard, World3DPlugin};
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
    for i in 1..7 {
        for j in 1..7 {
            commands.trigger(SpawnUnitEvent {
                position: Vec2::new(i as f32 * 40.0 + 40.0, j as f32 * 40.0 + 40.0),
                t_type: if i % 2 == 0 {
                    ThingType::AttackerGun
                } else {
                    ThingType::AttackerCannon
                },
                team: if (i + j) % 2 == 0 {
                    Team::Player
                } else {
                    Team::Enemy
                },
                hp: 100.0,
            });
        }
    }
}
