use bevy::prelude::*;
use chunk_flow_field::map::{CELL_BLOCKED, EDGE_BOTTOM, EDGE_RIGHT};

use crate::map::GameMap;

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

#[derive(Debug, Clone)]
pub enum CellInfo {
    Wall,
    Empty {
        obstacle_vec: Vec<Obstacle>,
        entity_vec: Vec<EntityInfo>,
    },
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
    pub map_size: Vec2,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<CellInfo>, // None Means it is Wall Cell
}

impl SpatialGrid {
    pub fn new(map: &GameMap) -> Self {
        let mut cells = vec![CellInfo::Wall; map.width * map.height];
        for (i, landform) in map.landforms.iter().enumerate() {
            if landform.blocked_mask & CELL_BLOCKED == 0 {
                let x = i % map.width;
                let y = i / map.width;
                let mut obstacle_vec = Vec::with_capacity(2);
                // Store each shared edge once, in cell-local coordinates.
                // Edges adjacent to Wall cells are already covered by their cuboids.
                if x + 1 < map.width
                    && landform.blocked_mask & EDGE_RIGHT != 0
                    && map.landforms[i + 1].blocked_mask & CELL_BLOCKED == 0
                {
                    obstacle_vec.push(Obstacle {
                        shape: parry2d::shape::SharedShape::segment(
                            parry2d::math::Vec2::new(map.cell_size, 0.0),
                            parry2d::math::Vec2::new(map.cell_size, map.cell_size),
                        ),
                        pose: parry2d::math::Pose::IDENTITY,
                    });
                }
                if y + 1 < map.height
                    && landform.blocked_mask & EDGE_BOTTOM != 0
                    && map.landforms[i + map.width].blocked_mask & CELL_BLOCKED == 0
                {
                    obstacle_vec.push(Obstacle {
                        shape: parry2d::shape::SharedShape::segment(
                            parry2d::math::Vec2::new(0.0, map.cell_size),
                            parry2d::math::Vec2::new(map.cell_size, map.cell_size),
                        ),
                        pose: parry2d::math::Pose::IDENTITY,
                    });
                }
                cells[i] = CellInfo::Empty {
                    obstacle_vec,
                    entity_vec: Vec::with_capacity(16),
                };
            }
        }
        Self {
            map_size: Vec2::new(
                map.width as f32 * map.cell_size,
                map.height as f32 * map.cell_size,
            ),
            cell_size: map.cell_size,
            width: map.width,
            height: map.height,
            cells,
        }
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

    pub fn get_entities_at(&self, position: Vec2) -> Option<Vec<EntityInfo>> {
        let (grid_x, grid_y) = self.world_to_grid(position)?;
        if grid_x >= self.width || grid_y >= self.height {
            return None; // 그리드 범위를 벗어남
        }
        if let CellInfo::Empty {
            obstacle_vec: _,
            entity_vec,
        } = &self.cells[grid_y * self.width + grid_x]
        {
            let mut rtn = Vec::new();
            for e in entity_vec {
                if (position - e.pos).length_squared() < e.radius.powi(2) {
                    rtn.push(e.clone());
                }
            }
            if rtn.len() > 0 {
                return Some(rtn);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    // Raycasting is not implemented for edge detection
    // pub fn raycast(&self, ray: &Ray, exclude: Option<&[Entity]>) -> RaycastResult {
    //     if ray.length <= 0.0 {
    //         return RaycastResult::Miss;
    //     }
    //     let (mut grid_x, mut grid_y) = if let Some(grid) = self.world_to_grid(ray.origin) {
    //         if let Some(vec) = self.cells[self.width * grid.1 + grid.0].as_ref() {
    //             let mut len_current = ray.length;
    //             let mut rtn = None;
    //             for info in vec {
    //                 if let Some(ex) = exclude {
    //                     if ex.contains(&info.entity) {
    //                         continue;
    //                     }
    //                 }
    //                 if let Some(t) = circle_line_overlap(
    //                     info.radius,
    //                     info.pos,
    //                     ray.origin,
    //                     ray.origin + ray.direction * ray.length,
    //                 ) {
    //                     if t * ray.length < len_current {
    //                         len_current = t * ray.length;
    //                         rtn = Some(info.clone());
    //                     }
    //                 }
    //             }
    //             if rtn.is_some() {
    //                 return RaycastResult::HitEntity(
    //                     ray.origin + ray.direction * len_current,
    //                     rtn.unwrap(),
    //                 );
    //             }
    //         } else {
    //             return RaycastResult::HitWall(ray.origin, Vec2::ZERO); // 시작점이 벽에 있음
    //         }
    //         grid
    //     } else {
    //         return RaycastResult::OutOfBounds; // 시작점이 그리드 범위를 벗어남
    //     };
    //     let step_x = ray.direction.x.signum() as isize;
    //     let step_y = ray.direction.y.signum() as isize;

    //     let ray_offset = ray.origin
    //         - Vec2::new(
    //             grid_x as f32 * self.cell_size,
    //             grid_y as f32 * self.cell_size,
    //         );
    //     let mut total_x = if ray.direction.x.abs() > 0.0001 {
    //         if ray.direction.x > 0.0 {
    //             (self.cell_size - ray_offset.x) / ray.direction.x
    //         } else {
    //             ray_offset.x / -ray.direction.x
    //         }
    //     } else {
    //         f32::MAX
    //     };
    //     let mut total_y = if ray.direction.y.abs() > 0.0001 {
    //         if ray.direction.y > 0.0 {
    //             (self.cell_size - ray_offset.y) / ray.direction.y
    //         } else {
    //             ray_offset.y / -ray.direction.y
    //         }
    //     } else {
    //         f32::MAX
    //     };
    //     let delta_x = if ray.direction.x.abs() > 0.0001 {
    //         self.cell_size / ray.direction.x.abs()
    //     } else {
    //         f32::MAX
    //     };
    //     let delta_y = if ray.direction.y.abs() > 0.0001 {
    //         self.cell_size / ray.direction.y.abs()
    //     } else {
    //         f32::MAX
    //     };
    //     let mut len_last;
    //     while total_x.min(total_y) < ray.length {
    //         len_last = total_x.min(total_y);
    //         if total_x < total_y {
    //             let next_x = grid_x as isize + step_x;
    //             if next_x >= self.width as isize || next_x < 0 {
    //                 return RaycastResult::OutOfBounds;
    //             }
    //             grid_x = next_x as usize;
    //             total_x += delta_x;
    //         } else {
    //             let next_y = grid_y as isize + step_y;
    //             if next_y >= self.height as isize || next_y < 0 {
    //                 return RaycastResult::OutOfBounds;
    //             }
    //             grid_y = next_y as usize;
    //             total_y += delta_y;
    //         }
    //         let mut len_current = total_x.min(total_y);
    //         let mut rtn = None;
    //         if let Some(vec) = self
    //             .cells
    //             .get(self.width * grid_y + grid_x)
    //             .and_then(|c| c.as_ref())
    //         {
    //             for info in vec {
    //                 if let Some(ex) = exclude {
    //                     if ex.contains(&info.entity) {
    //                         continue;
    //                     }
    //                 }
    //                 if let Some(t) = circle_line_overlap(
    //                     info.radius,
    //                     info.pos,
    //                     ray.origin,
    //                     ray.origin + ray.direction * ray.length,
    //                 ) {
    //                     if t * ray.length < len_current {
    //                         len_current = t * ray.length;
    //                         rtn = Some(info.clone());
    //                     }
    //                 }
    //             }
    //         } else {
    //             let normal = if total_x < total_y {
    //                 Vec2::new(-step_x as f32, 0.0)
    //             } else {
    //                 Vec2::new(0.0, -step_y as f32)
    //             };
    //             return RaycastResult::HitWall(ray.origin + ray.direction * len_last, normal);
    //         }
    //         if rtn.is_some() {
    //             return RaycastResult::HitEntity(
    //                 ray.origin + ray.direction * len_current,
    //                 rtn.unwrap(),
    //             );
    //         }
    //     }
    //     RaycastResult::Miss
    // }

    pub fn collision_check(
        &self,
        position: Vec2,
        radius: f32,
        exclude: Option<&[Entity]>,
        margin: f32,
    ) -> CollisionResult {
        let grid_radius = (radius / self.cell_size).ceil() as isize + 1;
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
                        match  &self.cells[idx] {
                            CellInfo::Empty {
                                obstacle_vec,
                                entity_vec,
                            } => {
                                for entity in entity_vec {
                                    if let Some(ex) = exclude {
                                        if ex.contains(&entity.entity) {
                                            continue;
                                        }
                                    }
                                    if (position - entity.pos).length_squared() < (radius + entity.radius + margin).powi(2) {
                                        collided_entities.push(entity.clone());
                                    }
                                }
                                for obstacle in obstacle_vec {
                                    let mut obstacle_moved = obstacle.clone();
                                    obstacle_moved.pose.translation +=
                                    parry2d::math::Vec2::new(x as f32 * self.cell_size, y as f32 * self.cell_size);
                                    obstacles.push(obstacle_moved);
                                }
                            },
                            CellInfo::Wall => {
                                let obstacle = Obstacle {
                                    shape: parry2d::shape::SharedShape::cuboid(self.cell_size / 2.0, self.cell_size / 2.0).into(),
                                    pose: parry2d::math::Pose::new(
                                        parry2d::math::Vec2::new(cell_x + self.cell_size / 2.0, cell_y + self.cell_size / 2.0), 0.0),
                                };
                                obstacles.push(obstacle);
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
        if idx < self.cells.len() {
            if let CellInfo::Empty {
                obstacle_vec: _,
                entity_vec,
            } = &mut self.cells[idx]
            {
                entity_vec.push(EntityInfo {
                    entity,
                    radius,
                    pos,
                });
            }
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            if let CellInfo::Empty {
                obstacle_vec: _,
                entity_vec,
            } = cell
            {
                entity_vec.clear();
            }
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
        let radius_in_cells = (radius / self.cell_size).ceil() as isize;

        for y in (grid_y as isize - radius_in_cells)..=(grid_y as isize + radius_in_cells) {
            for x in (grid_x as isize - radius_in_cells)..=(grid_x as isize + radius_in_cells) {
                if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                    let idx = (y as usize) * self.width + (x as usize);
                    if let CellInfo::Empty {
                        obstacle_vec: _,
                        entity_vec,
                    } = &self.cells[idx]
                    {
                        result.extend(
                            entity_vec
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
        }
        Ok(result)
    }

    pub fn query_entities_rect(&self, min: Vec2, max: Vec2) -> anyhow::Result<Vec<EntityInfo>> {
        let mut result = Vec::new();
        let (grid_min_x, grid_min_y) = self
            .world_to_grid(min)
            .ok_or_else(|| anyhow::anyhow!("Query min position is out of grid bounds"))?;
        let (grid_max_x, grid_max_y) = self
            .world_to_grid(max)
            .ok_or_else(|| anyhow::anyhow!("Query max position is out of grid bounds"))?;

        for y in grid_min_y..=grid_max_y {
            for x in grid_min_x..=grid_max_x {
                if x < self.width && y < self.height {
                    let idx = y * self.width + x;
                    if let CellInfo::Empty {
                        obstacle_vec: _,
                        entity_vec,
                    } = &self.cells[idx]
                    {
                        result.extend(entity_vec.iter().cloned());
                    }
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
                    if matches!(self.cells[idx], CellInfo::Wall) {
                        result.push((x as usize, y as usize));
                    }
                }
            }
        }
        Ok(result)
    }
}

fn circle_line_overlap(radius: f32, center: Vec2, line_start: Vec2, line_end: Vec2) -> Option<f32> {
    let delta = line_start - line_end;
    let sigma = 0.00001;
    let a = delta.length_squared();
    if a < sigma {
        return None;
    }
    let to_center = line_end - center;
    let b_half = to_center.x * delta.x + to_center.y * delta.y;
    let c = to_center.length_squared() - radius.powi(2);
    let det = b_half.powi(2) - a * c;
    if det < 0.0 {
        return None;
    }
    let t1 = (-b_half + det.sqrt()) / a;
    let t2 = (-b_half - det.sqrt()) / a;
    let t1_valid = t1 >= 0.0 && t1 <= 1.0;
    let t2_valid = t2 >= 0.0 && t2 <= 1.0;
    if t1_valid && t2_valid {
        return Some(t1.min(t2));
    } else if t1_valid {
        return Some(t1);
    } else if t2_valid {
        return Some(t2);
    } else {
        return None;
    }
}
