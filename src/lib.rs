pub mod combat;
pub mod constants;
pub mod debug;
pub mod input;
pub mod map;
pub mod movement;
pub mod thing;
pub mod ui;
pub mod unit;
pub mod visual;
pub mod world3d;
pub mod player;

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
use crate::{thing::ThingType, unit::{component::Team, event::SpawnUnitEvent}};

pub fn setup(mut commands: Commands) {
    for i in 1..7 {
        for j in 1..7 {
            commands.trigger(SpawnUnitEvent {
                position: Vec2::new(i as f32 * 40.0 + 200.0, j as f32 * 40.0 + 200.0),
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
