use bevy::prelude::*;

pub mod component;
pub mod event;
pub mod system;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
    }
}
