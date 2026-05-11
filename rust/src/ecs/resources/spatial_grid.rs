use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vector2,
    pub direction: Vector2,
    pub length: f32,
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub entity: Entity,
    pub radius: f32,
    pub pos: Vector2,
}

#[derive(Debug, Clone)]
pub enum RaycastResult {
    HitWall(Vector2, Vector2), // 벽에 맞은 위치와 벽의 법선 벡터
    HitEntity(Vector2, EntityInfo),
    Miss,
    OutOfBounds,
}

#[derive(Resource, Debug)]
pub struct SpatialGrid {
    cell_size: f32,
    width: usize,
    height: usize,
    cells: Vec<Option<Vec<EntityInfo>>>, // None Means it is Wall Cell
}

impl SpatialGrid {
    pub fn new(map_width: f32, map_height: f32, cell_size: f32, walls: &[bool]) -> Self {
        let width = (map_width / cell_size).ceil() as usize;
        let height = (map_height / cell_size).ceil() as usize;
        let mut cells = vec![None; width * height];
        for (i, &is_wall) in walls.iter().enumerate() {
            if !is_wall {
                cells[i] = Some(Vec::with_capacity(16));
            }
        }
        Self {
            cell_size,
            width,
            height,
            cells,
        }
    }

    pub fn world_to_grid(&self, position: Vector2) -> Option<(usize, usize)> {
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

    pub fn get_entities_at(&self, position: Vector2) -> Option<Vec<EntityInfo>> {
        let (grid_x, grid_y) = self.world_to_grid(position)?;
        if grid_x >= self.width || grid_y >= self.height {
            return None; // 그리드 범위를 벗어남
        }
        if let Some(entity_vec) = &self.cells[grid_y * self.width + grid_x] {
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

    pub fn raycast(&self, ray: &Ray, exclude: Option<&[Entity]>) -> RaycastResult {
        if ray.length <= 0.0 {
            return RaycastResult::Miss;
        }
        let (mut grid_x, mut grid_y) = if let Some(grid) = self.world_to_grid(ray.origin) {
            if let Some(vec) = self.cells[self.width * grid.1 + grid.0].as_ref() {
                let mut len_current = ray.length;
                let mut rtn = None;
                for info in vec {
                    if let Some(ex) = exclude {
                        if ex.contains(&info.entity) {
                            continue;
                        }
                    }
                    if let Some(t) = circle_line_overlap(
                        info.radius,
                        info.pos,
                        ray.origin,
                        ray.origin + ray.direction * ray.length,
                    ) {
                        if t * ray.length < len_current {
                            len_current = t * ray.length;
                            rtn = Some(info.clone());
                        }
                    }
                }
                if rtn.is_some() {
                    return RaycastResult::HitEntity(
                        ray.origin + ray.direction * len_current,
                        rtn.unwrap(),
                    );
                }
            } else {
                return RaycastResult::HitWall(ray.origin, Vector2::ZERO); // 시작점이 벽에 있음
            }
            grid
        } else {
            return RaycastResult::OutOfBounds; // 시작점이 그리드 범위를 벗어남
        };
        let step_x = ray.direction.x.signum() as isize;
        let step_y = ray.direction.y.signum() as isize;

        let ray_offset = ray.origin
            - Vector2::new(
                grid_x as f32 * self.cell_size,
                grid_y as f32 * self.cell_size,
            );
        let mut total_x = if ray.direction.x.abs() > 0.0001 {
            if ray.direction.x > 0.0 {
                (self.cell_size - ray_offset.x) / ray.direction.x
            } else {
                ray_offset.x / -ray.direction.x
            }
        } else {
            f32::MAX
        };
        let mut total_y = if ray.direction.y.abs() > 0.0001 {
            if ray.direction.y > 0.0 {
                (self.cell_size - ray_offset.y) / ray.direction.y
            } else {
                ray_offset.y / -ray.direction.y
            }
        } else {
            f32::MAX
        };
        let delta_x = if ray.direction.x.abs() > 0.0001 { self.cell_size / ray.direction.x.abs() } else { f32::MAX };
        let delta_y = if ray.direction.y.abs() > 0.0001 { self.cell_size / ray.direction.y.abs() } else { f32::MAX };
        let mut len_last = 0.0;
        while total_x.min(total_y) < ray.length {
            len_last = total_x.min(total_y);
            if total_x < total_y {
                let next_x = grid_x as isize + step_x;
                if next_x >= self.width as isize || next_x < 0 {
                    return RaycastResult::OutOfBounds;
                }
                grid_x = next_x as usize;
                total_x += delta_x;
            } else {
                let next_y = grid_y as isize + step_y;
                if next_y >= self.height as isize || next_y < 0 {
                    return RaycastResult::OutOfBounds;
                }
                grid_y = next_y as usize;
                total_y += delta_y;
            }
            let mut len_current = total_x.min(total_y);
            let mut rtn = None;
            if let Some(vec) = self.cells.get(self.width * grid_y + grid_x).and_then(|c| c.as_ref()) {
                for info in vec {
                    if let Some(ex) = exclude {
                        if ex.contains(&info.entity) {
                            continue;
                        }
                    }
                    if let Some(t) = circle_line_overlap(
                        info.radius,
                        info.pos,
                        ray.origin,
                        ray.origin + ray.direction * ray.length,
                    ) {
                        if t * ray.length < len_current {
                            len_current = t * ray.length;
                            rtn = Some(info.clone());
                        }
                    }
                }
            } else {
                let normal = if total_x < total_y {
                    Vector2::new(-step_x as f32, 0.0)
                } else {
                    Vector2::new(0.0, -step_y as f32)
                };
                return RaycastResult::HitWall(ray.origin + ray.direction * len_last, normal);
            }
            if rtn.is_some() {
                return RaycastResult::HitEntity(
                    ray.origin + ray.direction * len_current,
                    rtn.unwrap(),
                );
            }
        }
        RaycastResult::Miss
    }

    pub fn get_push_out_vector(&self, position: Vector2) -> Vector2 {
        let (grid_x, grid_y) = match self.world_to_grid(position) {
            Some(g) => g,
            None => return Vector2::ZERO,
        };

        if self.cells[grid_y * self.width + grid_x].is_some() {
            return Vector2::ZERO; // 벽이 아님
        }

        // 벽 내부인 경우: 주변 빈 셀 탐색
        let mut push_vec = Vector2::ZERO;
        let mut min_dist_sq = f32::MAX;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = grid_x as isize + dx;
                let ny = grid_y as isize + dy;

                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    if self.cells[ny as usize * self.width + nx as usize].is_some() {
                        let cell_center = Vector2::new((nx as f32 + 0.5) * self.cell_size, (ny as f32 + 0.5) * self.cell_size);
                        let to_center = cell_center - position;
                        let dist_sq = to_center.length_squared();
                        if dist_sq < min_dist_sq {
                            min_dist_sq = dist_sq;
                            push_vec = to_center.normalized_or_zero();
                        }
                    }
                }
            }
        }
        push_vec
    }

    pub fn add_entity(&mut self, entity: Entity, pos: Vector2, radius: f32) -> anyhow::Result<()> {
        let (grid_x, grid_y) = self
            .world_to_grid(pos)
            .ok_or_else(|| anyhow::anyhow!("Entity position is out of grid bounds"))?;
        let idx = grid_y * self.width + grid_x;
        if idx < self.cells.len() {
            if let Some(cell) = &mut self.cells[idx] {
                cell.push(EntityInfo {
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
            if let Some(entities) = cell {
                entities.clear();
            }
        }
    }

    pub fn query_entities(
        &self,
        position: Vector2,
        radius: f32,
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
                    if let Some(entities) = &self.cells[idx] {
                        result.extend(entities.iter().cloned());
                    }
                }
            }
        }
        Ok(result)
    }
}

fn circle_line_overlap(
    radius: f32,
    center: Vector2,
    line_start: Vector2,
    line_end: Vector2,
) -> Option<f32> {
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
