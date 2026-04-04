pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::ecs::components::{
        Dead, FlowField, FollowingOrder, MoveOrder, Transform, TransformID, UnitMovement,
    };
    pub use crate::ecs::events::{
        move_order_trigger, spawn_units_trigger, MoveOrderEvent, SpawnUnitEvent,
    };
    pub use crate::ecs::resources::{FlowGrid, Time, TransformBuffer};
    pub use crate::ecs::systems::{
        framework::{despawn_units_system, transform_update_system},
        movement::{cleanup_orders_system, movement_system, update_flow_field_system},
    };
}
