use bevy::prelude::*;
use chunk_flow_field::map::BuildingId;

use crate::unit::component::Position;
use serde::{Deserialize, Serialize};

mod placement;
pub use placement::{BeginBuildingPlacement, BuildingPlacementState};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct BuildingStats {
    pub size: UVec2,
}

impl BuildingStats {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.size.x > 0 && self.size.y > 0, "Building size must be nonzero");
        Ok(())
    }
}

/// Links a building entity to its registered obstacle in the pathfinding map.
#[derive(Component, Debug)]
#[require(Position)]
pub struct Building {
    pub id: BuildingId,
    /// Footprint width and height in FlowGrid cells, independent of sprite size.
    pub size: UVec2,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BuildingPlacementSet;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingPlacementState>()
            .add_observer(placement::begin_placement)
            .add_observer(placement::remove_building_obstacle)
            .add_systems(
                Update,
                (placement::update_placement, placement::update_building_transforms)
                    .chain()
                    .in_set(BuildingPlacementSet)
                    .after(crate::input::mouse_input)
                    .after(crate::world3d::rts_camera_system),
            )
            .add_systems(FixedPreUpdate, placement::sync_spatial_obstacles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunk_flow_field::map::{Map, Obstacle};

    #[test]
    fn building_stats_reject_zero_dimensions() {
        for size in [UVec2::ZERO, UVec2::new(0, 3), UVec2::new(3, 0)] {
            assert!(BuildingStats { size }.validate().is_err());
        }
        assert!(BuildingStats { size: UVec2::splat(3) }.validate().is_ok());
    }

    fn building() -> Building {
        let mut map = Map::new(&Obstacle::new(4, 3));
        let mut obstacle = Obstacle::new(4, 3);
        obstacle.set_cell_blocked(1, 1);
        Building {
            id: map.add_building(obstacle).unwrap(),
            size: UVec2::new(1, 1),
        }
    }

    #[test]
    fn building_requires_a_default_position() {
        let mut world = World::new();
        let entity = world.spawn(building()).id();
        assert_eq!(world.get::<Position>(entity).unwrap().0, Vec2::ZERO);
        assert_eq!(world.get::<Building>(entity).unwrap().size, UVec2::ONE);
    }

    #[test]
    fn building_preserves_an_explicit_position() {
        let mut world = World::new();
        let position = Vec2::new(100.0, 200.0);
        let entity = world.spawn((building(), Position(position))).id();
        assert_eq!(world.get::<Position>(entity).unwrap().0, position);
    }
}
