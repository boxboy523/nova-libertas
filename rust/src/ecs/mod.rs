pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::ecs::components::{Dead, Transform, TransformID, UnitStats};
    pub use crate::ecs::events::{spawn_units_trigger, SpawnUnitEvent};
    pub use crate::ecs::resources::{Time, TransformBuffer};
    pub use crate::ecs::systems::{
        framework::{despawn_units_system, transform_update_system},
        movement::movement_system,
    };
}
