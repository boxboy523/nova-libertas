pub mod combat;
pub mod constants;
pub mod movement;
pub mod unit;

pub mod prelude {
    pub use crate::ecs::combat::{
        component::{Attack, AutoAttack, UnitBattleStats, UnitHp},
        event::{attack_order_trigger, damage_trigger, AttackOrderEvent, DamageEvent},
        system::{attack_system, auto_attack_system, move_or_attack_system},
    };
    pub use crate::ecs::constants::*;
    pub use crate::ecs::movement::{
        component::{
            DelayedStopTrigger, FieldFollowTarget, FlowField, Moving, Stopped, UnitMovement,
        },
        event::{move_order_trigger, MoveOrderEvent},
        flow_grid::FlowGrid,
        system::{
            acceleration_system, apply_move_system, avoid_system, delayed_stop_system,
            flow_field_added_system, flow_movement_system, remove_empty_orders_system,
            smooth_wall_passing_system, stop_attacking_unit_system, stop_moving_unit_system,
            stopped_in_range_system, update_flow_field_system,
        },
    };
    pub use crate::ecs::unit::{
        component::{Dead, Selected, Team, Transform, UnitStats},
        event::{
            despawn_order_trigger, spawn_units_trigger, spawn_wall_trigger, SpawnUnitEvent,
            SpawnWallEvent,
        },
        spatial_grid::{CollisionResult, SpatialGrid},
        system::{despawn_units_system, transform_update_system, update_spatial_grid_system},
        transform_buffer::{ThingType, TransformBuffer, STRIDE},
        Time,
    };
}
