use bevy::prelude::*;
use chunk_flow_field::map::BuildingId;

use crate::unit::component::Position;

/// Links a building entity to its registered obstacle in the pathfinding map.
#[derive(Component, Debug)]
#[require(Position)]
pub struct Building {
    pub id: BuildingId,
    /// Footprint width and height in FlowGrid cells, independent of sprite size.
    pub size: UVec2,
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunk_flow_field::map::{Map, Obstacle};

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
