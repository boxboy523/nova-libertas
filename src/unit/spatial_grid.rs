use bevy::prelude::*;
use chunk_flow_field::{map::{EDGE_BOTTOM, EDGE_RIGHT}, types::Cell};

use crate::{movement::flow_grid::FlowGrid, util::circle_rect_overlap};

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec2,
    pub direction: Vec2,
    pub length: f32,
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub shape: parry2d::shape::SharedShape,
    pub pose: parry2d::math::Pose,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityInfo {
    pub entity: Entity,
    pub radius: f32,
    pub pos: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionInfo {
    pub distance: f32,
    pub normal: Vec2,
}

#[derive(Debug, Clone)]
pub enum RaycastResult {
    HitWall(Vec2, Vec2), // 벽에 맞은 위치와 벽의 법선 벡터
    HitEntity(Vec2, EntityInfo),
    Miss,
    OutOfBounds,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CollisionResult {
    Collided(Vec<EntityInfo>, Vec<CollisionInfo>), // Collided entites, Normal vectors of obstacles
    NoCollision,
    OutOfBounds,
}

#[derive(Resource, Debug)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub map_size: UVec2,
    pub width: usize,
    pub height: usize,
    pub obstacle_cells: Vec<Option<Vec<Obstacle>>>, // None for Wall Cells
    pub units: Vec<Vec<EntityInfo>>,
    map_generation: usize,
    max_unit_radius_cells: usize,
}

impl SpatialGrid {
    pub fn new(flow_grid: &FlowGrid) -> Self {
        let map = &flow_grid.map;
        let cell_size = flow_grid.cell_size;
        let mut grid = Self {
            map_size: UVec2::new(
                map.width as u32,
                map.height as u32,
            ),
            cell_size,
            width: map.width,
            height: map.height,
            obstacle_cells: Vec::new(),
            units: vec![Vec::with_capacity(16); map.width * map.height],
            map_generation: 0,
            max_unit_radius_cells: 0,
        };
        grid.rebuild_obstacles(flow_grid);
        grid
    }

    /// Refresh obstacles from the same map when its generation changes.
    /// Map dimensions and cell size must remain unchanged; unit entries are preserved.
    pub fn update(&mut self, flow_grid: &FlowGrid) {
        if self.map_generation != flow_grid.map.get_generation() {
            self.rebuild_obstacles(flow_grid);
        }
    }

    fn rebuild_obstacles(&mut self, flow_grid: &FlowGrid) {
        let map = &flow_grid.map;
        let cell_size = flow_grid.cell_size;
        let mut obstacle_cells = vec![None; map.width * map.height];
        for y in 0..map.height {
            for x in 0..map.width {
                if matches!(map.get_cell(x, y), Cell::Wall) {
                    continue;
                }
                let mut obstacle_vec = Vec::with_capacity(2);
                // Store each shared edge once, in cell-local coordinates.
                // Edges adjacent to Wall cells are already covered by their cuboids.
                if x + 1 < map.width
                    && !map.can_pass(x, y, EDGE_RIGHT)
                    && !matches!(map.get_cell(x + 1, y), Cell::Wall)
                {
                    obstacle_vec.push(Obstacle {
                        shape: parry2d::shape::SharedShape::segment(
                            parry2d::math::Vec2::new(cell_size, 0.0),
                            parry2d::math::Vec2::new(cell_size, cell_size),
                        ),
                        pose: parry2d::math::Pose::IDENTITY,
                    });
                }
                if y + 1 < map.height
                    && !map.can_pass(x, y, EDGE_BOTTOM)
                    && !matches!(map.get_cell(x, y + 1), Cell::Wall)
                {
                    obstacle_vec.push(Obstacle {
                        shape: parry2d::shape::SharedShape::segment(
                            parry2d::math::Vec2::new(0.0, cell_size),
                            parry2d::math::Vec2::new(cell_size, cell_size),
                        ),
                        pose: parry2d::math::Pose::IDENTITY,
                    });
                }
                obstacle_cells[y * map.width + x] = Some(obstacle_vec);
            }
        }
        self.obstacle_cells = obstacle_cells;
        self.map_generation = map.get_generation();
    }

    pub fn world_to_grid(&self, position: Vec2) -> Option<(usize, usize)> {
        if position.x < 0.0 || position.y < 0.0 {
            return None; // 음수 좌표는 그리드 범위를 벗어남
        }
        let x = (position.x / self.cell_size).floor() as usize;
        let y = (position.y / self.cell_size).floor() as usize;
        if x < self.width && y < self.height {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn grid_to_world(&self, grid_pos: Vec2) -> Vec2 {
        Vec2::new(
            (grid_pos.x + 0.5) * self.cell_size,
            (grid_pos.y + 0.5) * self.cell_size,
        )
    }

    pub fn get_entities_at(&self, position: Vec2) -> anyhow::Result<Vec<EntityInfo>> {
        let (grid_x, grid_y) = self
            .world_to_grid(position)
            .ok_or_else(|| anyhow::anyhow!("Query position is out of grid bounds"))?;
        let idx = grid_y * self.width + grid_x;
        if self.obstacle_cells[idx].is_none() {
            anyhow::bail!("Query position is in a wall cell");
        }
        Ok(self.units[idx]
            .iter()
            .filter(|e| (position - e.pos).length_squared() < e.radius.powi(2))
            .cloned()
            .collect())
    }

    pub fn collision_check(
        &self,
        position: Vec2,
        radius: f32,
        exclude: Option<&[Entity]>,
        margin: f32,
    ) -> CollisionResult {
        let grid_radius = (radius / self.cell_size).ceil() as isize + self.max_unit_radius_cells as isize;
        if let Some((grid_x, grid_y)) = self.world_to_grid(position) {
            let mut collided_entities = Vec::new();
            let mut obstacles = Vec::new();
            let mut collision_infos = Vec::new();
            for y in (grid_y as isize - grid_radius)..=(grid_y as isize + grid_radius) {
                for x in (grid_x as isize - grid_radius)..=(grid_x as isize + grid_radius) {
                    if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                        let cell_x = x as f32 * self.cell_size;
                        let cell_y = y as f32 * self.cell_size;
                        let idx = (y as usize) * self.width + (x as usize);
                        for entity in &self.units[idx] {
                            if exclude.is_some_and(|excluded| excluded.contains(&entity.entity)) {
                                continue;
                            }
                            if (position - entity.pos).length_squared()
                                < (radius + entity.radius + margin).powi(2)
                            {
                                collided_entities.push(entity.clone());
                            }
                        }
                        match &self.obstacle_cells[idx] {
                            Some(obstacle_vec) => {
                                for obstacle in obstacle_vec {
                                    let mut obstacle_moved = obstacle.clone();
                                    obstacle_moved.pose.translation +=
                                        parry2d::math::Vec2::new(cell_x, cell_y);
                                    obstacles.push(obstacle_moved);
                                }
                            }
                            None => {
                                obstacles.push(Obstacle {
                                    shape: parry2d::shape::SharedShape::cuboid(
                                        self.cell_size / 2.0, self.cell_size / 2.0,
                                    ),
                                    pose: parry2d::math::Pose::new(
                                        parry2d::math::Vec2::new(
                                            cell_x + self.cell_size / 2.0,
                                            cell_y + self.cell_size / 2.0,
                                        ),
                                        0.0,
                                    ),
                                });
                            }
                        }
                        for obstacle in &obstacles {
                            let unit_shape = parry2d::shape::Ball::new(radius);
                            let unit_pose = parry2d::math::Pose::new(parry2d::math::Vec2::new(position.x, position.y), 0.0);
                            let Ok(result) = parry2d::query::contact(
                                &obstacle.pose,
                                obstacle.shape.as_ref(),
                                &unit_pose,
                                &unit_shape,
                                margin,
                            ) else {
                                warn!("Failed to compute contact between obstacle and unit");
                                continue;
                            };
                            if let Some(contact) = result {
                                let normal = contact.normal1;
                                collision_infos.push(CollisionInfo {
                                    distance: contact.dist,
                                    normal: Vec2::new(normal.x, normal.y),
                                });
                            }
                        }
                    }
                }
            }
            if !collided_entities.is_empty() || !collision_infos.is_empty() {
                return CollisionResult::Collided(collided_entities, collision_infos);
            }
        } else {
            return CollisionResult::OutOfBounds;
        }
        CollisionResult::NoCollision
    }

    pub fn add_entity(&mut self, entity: Entity, pos: Vec2, radius: f32) -> anyhow::Result<()> {
        let (grid_x, grid_y) = self
            .world_to_grid(pos)
            .ok_or_else(|| anyhow::anyhow!("Entity position is out of grid bounds"))?;
        let idx = grid_y * self.width + grid_x;
        self.units[idx].push(EntityInfo { entity, radius, pos });
        let radius_cells = (radius / self.cell_size).ceil() as usize;
        if radius_cells > self.max_unit_radius_cells {
            self.max_unit_radius_cells = radius_cells;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        for entities in &mut self.units {
            entities.clear();
        }
    }

    pub fn query_entities(
        &self,
        position: Vec2,
        radius: f32,
        by_center: bool,
    ) -> anyhow::Result<Vec<EntityInfo>> {
        let mut result = Vec::new();
        let (grid_x, grid_y) = self
            .world_to_grid(position)
            .ok_or_else(|| anyhow::anyhow!("Query position is out of grid bounds"))?;
        let radius_in_cells = (radius / self.cell_size).ceil() as isize + if by_center { 0 } else { self.max_unit_radius_cells as isize };

        for y in (grid_y as isize - radius_in_cells)..=(grid_y as isize + radius_in_cells) {
            for x in (grid_x as isize - radius_in_cells)..=(grid_x as isize + radius_in_cells) {
                if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                    let idx = (y as usize) * self.width + (x as usize);
                    result.extend(
                        self.units[idx]
                            .iter()
                            .filter(|e| {
                                (position - e.pos).length_squared()
                                    < (radius + if by_center { 0.0 } else { e.radius }).powi(2)
                            })
                            .cloned(),
                    );
                }
            }
        }
        Ok(result)
    }

    pub fn query_entities_rect(&self, min: Vec2, max: Vec2, by_center: bool) -> anyhow::Result<Vec<EntityInfo>> {
        let mut result = Vec::new();
        let (grid_min_x, grid_min_y) = self
            .world_to_grid(min)
            .ok_or_else(|| anyhow::anyhow!("Query min position is out of grid bounds"))?;
        let (grid_max_x, grid_max_y) = self
            .world_to_grid(max)
            .ok_or_else(|| anyhow::anyhow!("Query max position is out of grid bounds"))?;

        let search_min_x = grid_min_x.saturating_sub(if by_center { 0 } else { self.max_unit_radius_cells });
        let search_min_y = grid_min_y.saturating_sub(if by_center { 0 } else { self.max_unit_radius_cells });
        let search_max_x = (grid_max_x + if by_center { 0 } else { self.max_unit_radius_cells }).min(self.width - 1);
        let search_max_y = (grid_max_y + if by_center { 0 } else { self.max_unit_radius_cells }).min(self.height - 1);
        for y in search_min_y..=search_max_y {
            for x in search_min_x..=search_max_x {
                let idx = y * self.width + x;
                if grid_min_x < x && x < grid_max_x && grid_min_y < y && y < grid_max_y { // Entites in strictly inside the rectangle
                    result.extend(
                        self.units[idx].iter().cloned()
                    );
                } else if !by_center { // Boundery Case, detect overlapping
                    result.extend(
                        self.units[idx]
                            .iter()
                            .filter(|e| {
                                circle_rect_overlap(e.pos, e.radius, min, max)
                            })
                            .cloned(),
                    );
                } else {
                    result.extend(
                        self.units[idx]
                            .iter()
                            .filter(|e| {
                                e.pos.x >= min.x && e.pos.x <= max.x && e.pos.y >= min.y && e.pos.y <= max.y
                            })
                            .cloned(),
                    );
                }
            }
        }
        Ok(result)
    }

    pub fn query_walls(&self, position: Vec2, radius: f32) -> anyhow::Result<Vec<(usize, usize)>> {
        let mut result = Vec::new();
        let (grid_x, grid_y) = self
            .world_to_grid(position)
            .ok_or_else(|| anyhow::anyhow!("Query position is out of grid bounds"))?;
        let radius_in_cells = (radius / self.cell_size).ceil() as isize;

        for y in (grid_y as isize - radius_in_cells)..=(grid_y as isize + radius_in_cells) {
            for x in (grid_x as isize - radius_in_cells)..=(grid_x as isize + radius_in_cells) {
                if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                    let idx = (y as usize) * self.width + (x as usize);
                    if self.obstacle_cells[idx].is_none() {
                        result.push((x as usize, y as usize));
                    }
                }
            }
        }
        Ok(result)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_flow_map_obstacles_and_generation() {
        use chunk_flow_field::map::{Map, Obstacle as MapObstacle, Side};

        let mut terrain = MapObstacle::new(4, 3);
        terrain.set_cell_blocked(0, 0);
        terrain.set_edge(1, 1, Side::Right);
        terrain.set_edge(1, 1, Side::Bottom);
        let mut map = Map::new(&terrain);
        let mut building = MapObstacle::new(4, 3);
        building.set_cell_blocked(3, 2);
        let building_id = map.add_building(building).unwrap();
        let mut flow_grid = FlowGrid {
            width: 4,
            height: 3,
            cell_size: 20.0,
            map,
        };

        let grid = SpatialGrid::new(&flow_grid);
        assert_eq!(grid.map_generation, flow_grid.map.get_generation());
        assert_ne!(grid.map_generation, 0);
        assert_eq!((grid.width, grid.height), (4, 3));
        assert_eq!(grid.cell_size, 20.0);
        assert_eq!(grid.map_size, UVec2::new(4, 3));
        assert!(grid.obstacle_cells[0].is_none());
        assert!(grid.obstacle_cells[11].is_none());
        assert_eq!(grid.obstacle_cells[5].as_ref().unwrap().len(), 2);
        assert!(grid.units.iter().all(Vec::is_empty));
        assert_eq!(grid.units.len(), 12);

        flow_grid.map.remove_building(building_id).unwrap();
        let restored = SpatialGrid::new(&flow_grid);
        assert_eq!(restored.map_generation, flow_grid.map.get_generation());
        assert!(restored.obstacle_cells[11].is_some());
        assert!(restored.obstacle_cells[0].is_none());
        assert_eq!(restored.obstacle_cells[5].as_ref().unwrap().len(), 2);
    }

    #[test]
    fn update_refreshes_changed_obstacles_and_preserves_units() {
        use chunk_flow_field::map::{Map, Obstacle as MapObstacle};

        let mut flow_grid = FlowGrid {
            width: 4,
            height: 3,
            cell_size: 20.0,
            map: Map::new(&MapObstacle::new(4, 3)),
        };
        let mut grid = SpatialGrid::new(&flow_grid);
        assert_eq!(grid.map_generation, 0);
        assert!(grid.obstacle_cells.iter().all(Option::is_some));

        grid.add_entity(Entity::from_bits(1), Vec2::new(10.0, 10.0), 2.0)
            .unwrap();
        let units = grid.units.clone();
        // With no map change, update must leave the existing cache in place.
        let obstacles_ptr = grid.obstacle_cells.as_ptr();
        grid.update(&flow_grid);
        assert_eq!(grid.obstacle_cells.as_ptr(), obstacles_ptr);

        let mut building = MapObstacle::new(4, 3);
        building.set_cell_blocked(3, 2);
        let id = flow_grid.map.add_building(building).unwrap();
        grid.update(&flow_grid);
        assert!(grid.obstacle_cells[11].is_none());
        assert_eq!(grid.map_generation, flow_grid.map.get_generation());
        assert_eq!(grid.units, units);

        flow_grid.map.remove_building(id).unwrap();
        grid.update(&flow_grid);
        assert!(grid.obstacle_cells[11].is_some());
        assert_eq!(grid.map_generation, flow_grid.map.get_generation());
        assert_eq!(grid.units, units);
    }

    fn test_grid() -> SpatialGrid {
        SpatialGrid {
            cell_size: 20.0,
            map_size: UVec2::new(2, 1),
            width: 2,
            height: 1,
            obstacle_cells: vec![None, Some(Vec::new())],
            units: vec![Vec::new(), Vec::new()],
            max_unit_radius_cells: 0,
            map_generation: 0,
        }
    }

    #[test]
    fn registration_tracks_maximum_radius_in_cells() {
        let mut grid = test_grid();
        assert_eq!(grid.max_unit_radius_cells, 0);
        for (id, radius, expected) in [(1, 2.0, 1), (2, 21.0, 2), (3, 1.0, 2)] {
            grid.add_entity(Entity::from_bits(id), Vec2::splat(10.0), radius).unwrap();
            assert_eq!(grid.max_unit_radius_cells, expected);
        }
    }

    #[test]
    fn rectangle_overlap_finds_neighboring_unit_but_center_mode_excludes_it() {
        let mut grid = test_grid();
        let entity = Entity::from_bits(1);
        grid.add_entity(entity, Vec2::new(21.0, 10.0), 3.0).unwrap();
        let min = Vec2::new(10.0, 5.0);
        let max = Vec2::new(19.0, 15.0);
        assert!(grid.query_entities_rect(min, max, true).unwrap().is_empty());
        let overlaps = grid.query_entities_rect(min, max, false).unwrap();
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].entity, entity);
    }

    #[test]
    fn entities_are_indexed_independently_of_walls() {
        let mut grid = test_grid();
        let entity = Entity::from_bits(1);
        let position = Vec2::splat(10.0);
        grid.add_entity(entity, position, 2.0).unwrap();

        assert!(grid.get_entities_at(position).is_err());
        assert_eq!(grid.query_entities(position, 1.0, true).unwrap().len(), 1);
        assert_eq!(
            grid.query_entities_rect(Vec2::ZERO, Vec2::new(39.0, 19.0), false)
                .unwrap().len(),
            1,
        );
        assert!(matches!(
            grid.collision_check(position, 2.0, Some(&[entity]), 0.0),
            CollisionResult::Collided(entities, contacts)
                if entities.is_empty() && !contacts.is_empty()
        ));
    }

    #[test]
    fn get_entities_at_returns_matches_or_empty_in_walkable_cells() {
        let mut grid = test_grid();
        let position = Vec2::new(30.0, 10.0);
        assert!(grid.get_entities_at(position).unwrap().is_empty());

        let entity = Entity::from_bits(1);
        grid.add_entity(entity, position, 2.0).unwrap();
        assert_eq!(grid.get_entities_at(position).unwrap()[0].entity, entity);
        assert!(grid.get_entities_at(Vec2::new(35.0, 10.0)).unwrap().is_empty());
    }

    #[test]
    fn get_entities_at_rejects_walls_and_out_of_bounds() {
        let grid = test_grid();
        for position in [
            Vec2::splat(10.0),
            Vec2::new(-1.0, 10.0),
            Vec2::new(10.0, -1.0),
            Vec2::new(40.0, 10.0),
            Vec2::new(10.0, 20.0),
        ] {
            assert!(grid.get_entities_at(position).is_err(), "{position:?}");
        }
    }

    #[test]
    fn clear_preserves_obstacles_and_generation() {
        let mut grid = test_grid();
        grid.map_generation = 7;
        grid.add_entity(Entity::from_bits(1), Vec2::new(30.0, 10.0), 2.0)
            .unwrap();
        grid.clear();

        assert!(grid.units.iter().all(Vec::is_empty));
        assert!(grid.obstacle_cells[0].is_none());
        assert!(grid.obstacle_cells[1].as_ref().unwrap().is_empty());
        assert_eq!(grid.map_generation, 7);
        assert_eq!(grid.query_walls(Vec2::new(10.0, 10.0), 0.0).unwrap(), vec![(0, 0)]);
        assert!(grid.add_entity(Entity::from_bits(2), Vec2::new(-1.0, 0.0), 2.0).is_err());
    }
}
