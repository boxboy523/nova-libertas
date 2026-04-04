use bevy_ecs::prelude::*;

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
