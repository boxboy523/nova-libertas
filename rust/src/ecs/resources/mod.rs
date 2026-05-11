use bevy_ecs::prelude::*;

pub mod flow_grid;
pub mod spatial_grid;
pub mod transform_buffer;

#[derive(Resource, Debug)]
pub struct Time {
    pub delta: f32,
}
