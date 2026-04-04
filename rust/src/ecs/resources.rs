use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;

#[derive(Resource)]
pub struct TransformBuffer {
    pub data: Vec<f32>,
    pub free_indices: Vec<usize>,
}

const CHUNK_SIZE: usize = 256; // 버퍼 확장 시 한 번에 할당할 유닛 수

impl TransformBuffer {
    pub fn new(len: usize) -> Self {
        Self {
            data: vec![0.0; len * 8],
            free_indices: (0..len).collect(), // 초기에는 모든 인덱스가 사용 가능
        }
    }

    pub fn allocate(&mut self) -> usize {
        if let Some(id) = self.free_indices.pop() {
            id
        } else {
            let new_id = self.data.len() / 8;
            self.data.resize(self.data.len() + CHUNK_SIZE * 8, 0.0); // 버퍼 확장
            self.free_indices
                .extend((new_id + 1)..(new_id + CHUNK_SIZE)); // 새 인덱스 추가
            new_id
        }
    }

    pub fn free(&mut self, index: usize) {
        let offset = index * 8;
        for i in 0..8 {
            self.data[offset + i] = 0.0; // 해제된 인덱스의 데이터 초기화
        }
        self.free_indices.push(index);
    }
}

#[derive(Resource)]
pub struct Time {
    pub delta: f32,
}

#[derive(Resource)]
pub struct FlowGrid {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
}

impl FlowGrid {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
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

    pub fn vector_from_flow_field(&self, flow_field: &FlowField, position: Vector2) -> Vector2 {
        let (grid_pos_x, grid_pos_y) = self.world_to_grid(position);
        let index = (grid_pos_y as usize * self.width + grid_pos_x as usize) as usize;
        if index < flow_field.field.len() {
            flow_field.field[index]
        } else {
            Vector2::ZERO
        }
    }

    pub fn gen_flow_field(&self, target: Vector2) -> Vec<Vector2> {
        // 간단한 흐름 필드 생성 (실제 구현에서는 A* 알고리즘 등을 사용하여 최적화 필요)
        let mut field = vec![Vector2::ZERO; self.width * self.height];
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let pos = Vector2::new(x as f32 * self.cell_size, y as f32 * self.cell_size);
                field[idx] = (target - pos).normalized(); // 타겟 방향으로 벡터 계산
            }
        }
        field
    }
}
