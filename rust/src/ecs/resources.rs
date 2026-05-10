use std::collections::HashMap;

use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use chunk_flow_field::map::Map;
use godot::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThingType {
    Test,
    Wall,
}

pub const CHUNK_SIZE: usize = 256; // 버퍼 확장 시 한 번에 할당할 유닛 수
const EXPAND_SIZE: usize = 8; // 버퍼 확장 시 청크 수 (CHUNK_SIZE * EXPAND_SIZE 만큼 유닛 추가)

#[derive(Debug, Clone)]
pub struct Chunk {
    index: usize,
    pub length: usize,
    pub t_type: Option<ThingType>,
    pub entity_map: [Option<Entity>; CHUNK_SIZE],
    is_full: bool,
    pub modified: bool,
}

impl Chunk {
    pub fn new(index: usize) -> Self {
        Self {
            index: index,
            length: 0,
            t_type: None,
            entity_map: [None; CHUNK_SIZE],
            is_full: false,
            modified: false,
        }
    }
}

// ECS 시스템에서 유닛의 위치, 회전, 크기 등의 변환 정보를 관리하는 버퍼
#[derive(Resource)]
pub struct TransformBuffer {
    pub data: Vec<f32>,
    pub chunks: Vec<Chunk>,
    free_chunks: Vec<usize>, // 사용 가능한 청크 인덱스 목록
    free_thing_chunks: HashMap<ThingType, Vec<usize>>, // ThingType별로 사용 가능한 청크 인덱스 목록
}

impl TransformBuffer {
    pub fn new(len: usize) -> Self {
        let chunks = (0..len).map(|i| Chunk::new(i)).collect();
        Self {
            data: vec![0.0; len * 8 * CHUNK_SIZE], // 초기 버퍼 크기 (len * 8 floats per unit)
            chunks,
            free_chunks: (0..len).collect(), // 초기에는 모든 청크가 사용
            free_thing_chunks: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, t_type: ThingType, entity: Entity) -> usize {
        if let Some(index) = self
            .free_thing_chunks
            .get_mut(&t_type)
            .and_then(|chunks| chunks.last())
        {
            if let Some(chunk) = self.chunks.get_mut(*index) {
                chunk.entity_map[chunk.length] = Some(entity);
                chunk.length += 1;
                chunk.modified = true;
                if chunk.length >= CHUNK_SIZE {
                    chunk.is_full = true;
                    self.free_thing_chunks.get_mut(&t_type).unwrap().pop();
                }
                return chunk.index * CHUNK_SIZE + chunk.length - 1;
            } else {
                unreachable!("청크 인덱스가 유효하지 않습니다.");
            }
        } else {
            if let Some(chunk_index) = self.free_chunks.pop() {
                let chunk = &mut self.chunks[chunk_index];
                chunk.t_type = Some(t_type);
                chunk.is_full = false;
                self.free_thing_chunks
                    .entry(t_type)
                    .or_default()
                    .push(chunk_index);
                return self.allocate(t_type, entity);
            } else {
                self.data.extend(vec![0.0; CHUNK_SIZE * 8 * EXPAND_SIZE]); // 버퍼 확장
                for i in 0..EXPAND_SIZE {
                    let new_chunk_index = self.chunks.len() + i;
                    self.chunks.push(Chunk::new(new_chunk_index));
                    self.free_chunks.push(new_chunk_index);
                }
                return self.allocate(t_type, entity);
            }
        }
    }

    pub fn free(&mut self, index: usize) -> Entity {
        let chunk_index = index / CHUNK_SIZE;
        let unit_index = index % CHUNK_SIZE;
        if let Some(chunk) = self.chunks.get_mut(chunk_index) {
            if let Some(entity) = chunk.entity_map[unit_index] {
                chunk.length -= 1;
                chunk.entity_map[unit_index] = chunk.entity_map[chunk.length];
                chunk.entity_map[chunk.length] = None;
                let to_swap = self.data[(chunk_index * CHUNK_SIZE + chunk.length) * 8
                    ..(chunk_index * CHUNK_SIZE + chunk.length + 1) * 8]
                    .to_vec();
                self.data[index * 8..(index + 1) * 8].copy_from_slice(&to_swap);
                self.data[(chunk_index * CHUNK_SIZE + chunk.length) * 8
                    ..(chunk_index * CHUNK_SIZE + chunk.length + 1) * 8]
                    .fill(0.0);
                chunk.modified = true;
                if chunk.is_full {
                    chunk.is_full = false;
                    self.free_thing_chunks
                        .get_mut(&chunk.t_type.unwrap())
                        .unwrap()
                        .push(chunk_index);
                }
                if chunk.length == 0 {
                    chunk.t_type = None;
                    self.free_chunks.push(chunk_index);
                    if let Some(v) = self.free_thing_chunks.get_mut(&chunk.t_type.unwrap()) {
                        v.retain(|&i| i != chunk_index);
                    }
                }
                return entity;
            } else {
                unreachable!("해제하려는 인덱스에 엔티티가 없습니다.");
            }
        } else {
            unreachable!("해제하려는 인덱스가 유효하지 않습니다.");
        }
    }
}

#[derive(Resource)]
pub struct Time {
    pub delta: f32,
}

#[derive(Resource)]
pub struct FlowGrid {
    // grid 기반의 플로우 필드 시스템을 위한 리소스
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub map: Map,
}

impl FlowGrid {
    pub fn new(map_width: f32, map_height: f32, cell_size: f32, wall: &[bool]) -> Self {
        let width = (map_width / cell_size).ceil() as usize;
        let height = (map_height / cell_size).ceil() as usize;
        Self {
            width,
            height,
            cell_size,
            map: Map::new(width, height, wall),
        }
    }

    pub fn world_to_grid(&self, position: Vector2) -> (usize, usize) {
        let x = (position.x / self.cell_size).floor() as usize;
        let y = (position.y / self.cell_size).floor() as usize;
        (x.min(self.width - 1), y.min(self.height - 1)) // 그리드 범위 내로 제한
    }

    pub fn grid_to_world(&self, grid_pos: Vector2) -> Vector2 {
        Vector2::new(
            (grid_pos.x + 0.5) * self.cell_size,
            (grid_pos.y + 0.5) * self.cell_size,
        )
    }

    // 주어진 월드 좌표에서 플로우 필드 벡터를 샘플링하는 함수 (양선형 보간)
    pub fn sample_flow_field(&self, flow_field: &FlowField, position: Vector2) -> Option<Vector2> {
        let gx = position.x / self.cell_size;
        let gy = position.y / self.cell_size;
        let grid_x = gx.floor() as usize;
        let grid_y = gy.floor() as usize;
        let tx = gx - grid_x as f32;
        let ty = gy - grid_y as f32;
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let v00 = flow_field
            .field
            .get(grid_y * self.width + grid_x)?
            .as_ref()?;
        let v10 = flow_field
            .field
            .get(grid_y * self.width + (grid_x + 1))
            .and_then(|v| v.as_ref())
            .unwrap_or(v00);
        let v01 = flow_field
            .field
            .get((grid_y + 1) * self.width + grid_x)
            .and_then(|v| v.as_ref())
            .unwrap_or(v00);
        let v11 = flow_field
            .field
            .get((grid_y + 1) * self.width + (grid_x + 1))
            .and_then(|v| v.as_ref())
            .unwrap_or(v00);
        let vx = lerp(lerp(v00.x, v10.x, tx), lerp(v01.x, v11.x, tx), ty);
        let vy = lerp(lerp(v00.y, v10.y, tx), lerp(v01.y, v11.y, tx), ty);
        Some(Vector2::new(vx, vy))
    }

    pub fn gen_flow_field(&self, target: Vector2) -> anyhow::Result<Vec<Option<Vector2>>> {
        let eps = 0.001; // 작은 값으로 0 나누기 방지
        let to_vec2 = |(x, y)| {
            if x < eps && x > -eps && y < eps && y > -eps {
                Vector2::ZERO
            } else {
                Vector2::new(x as f32, y as f32)
            }
        };
        let goal = chunk_flow_field::types::Pos {
            x: (target.x / self.cell_size).floor() as usize,
            y: (target.y / self.cell_size).floor() as usize,
        };
        let field = self.map.build_flow_field(goal)?;
        Ok(field.flow.into_iter().map(|opt| opt.map(to_vec2)).collect())
    }
}

#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    width: usize,
    height: usize,
    cells: Vec<Option<Vec<Entity>>>, // None Means it is Wall Cell
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

    pub fn world_to_grid(&self, position: Vector2) -> (usize, usize) {
        let x = (position.x / self.cell_size).floor() as usize;
        let y = (position.y / self.cell_size).floor() as usize;
        (x.min(self.width - 1), y.min(self.height - 1)) // 그리드 범위 내로 제한
    }

    pub fn is_wall_collision(&self, pos: Vector2, size: (f32, f32)) -> usize {
        // 0: no collision, 1: x-collision, 2: y-collision, 3: both
        let mut collision = 0;
        let (grid_x, grid_y) = self.world_to_grid(pos);
        let (grid_x2, grid_y2) = self.world_to_grid(pos + Vector2::new(size.0, size.1));
        for y in grid_y..=grid_y2 {
            for x in grid_x..=grid_x2 {
                if x < self.width && y < self.height {
                    if self.cells[y * self.width + x].is_none() {
                        if x == grid_x {
                            collision |= 1; // x-collision
                        }
                        if y == grid_y {
                            collision |= 2; // y-collision
                        }
                    }
                }
            }
        }
        collision
    }

    pub fn add_entity(&mut self, entity: Entity, position: Vector2) {
        let (grid_x, grid_y) = self.world_to_grid(position);
        let idx = grid_y * self.width + grid_x;
        if idx < self.cells.len() {
            if let Some(cell) = &mut self.cells[idx] {
                cell.push(entity);
            }
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            if let Some(entities) = cell {
                entities.clear();
            }
        }
    }

    pub fn query_entities(&self, position: Vector2, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let (grid_x, grid_y) = self.world_to_grid(position);
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
        result
    }
}
