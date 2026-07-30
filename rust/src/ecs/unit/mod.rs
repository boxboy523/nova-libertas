pub mod component;
pub mod event;
pub mod spatial_grid;
pub mod system;
pub mod transform_buffer;

use bevy_ecs::prelude::*;
#[derive(Resource, Debug)]
pub struct Time {
    pub delta: f32,
}
