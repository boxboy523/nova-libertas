use bevy_ecs::prelude::*;
use std::collections::HashMap;

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
