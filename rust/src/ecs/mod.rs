pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::ecs::components::{
        Dead, FlowField, MoveOrder, Transform, TransformID, UnitMovement,
    };
    pub use crate::ecs::events::{
        move_order_trigger, spawn_units_trigger, spawn_wall_trigger, MoveOrderEvent,
        SpawnUnitEvent, SpawnWallEvent,
    };
    pub use crate::ecs::resources::{
        flow_grid::FlowGrid, spatial_grid::Ray, spatial_grid::RaycastResult,
        spatial_grid::SpatialGrid, transform_buffer::ThingType, transform_buffer::TransformBuffer,
        transform_buffer::CHUNK_SIZE, Time,
    };
    pub use crate::ecs::systems::{
        framework::{despawn_units_system, transform_update_system},
        movement::{
            acceleration_system, apply_move_system, flow_movement_system, seperation_force_system,
            update_flow_field_system, update_spatial_grid_system,
        },
    };
}
