use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::{global::godot_print, meta::GodotConvert};
use std::collections::HashMap;
use strum::EnumIter;

#[derive(GodotConvert, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
#[godot(via = i64)]
pub enum ThingType {
    Test,
    Wall,
}

// ECS 시스템에서 유닛의 위치, 회전, 크기 등의 변환 정보를 관리하는 버퍼
#[derive(Resource)]
pub struct TransformBuffer {
    pub data: HashMap<ThingType, Vec<f32>>,
    pub entity_map: HashMap<ThingType, Vec<Entity>>, // buffer_index -> Entity
}

pub struct DeleteInfo {
    pub swapped_entity: Entity,
    pub swapped_index: usize,
}

impl TransformBuffer {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            entity_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, mut transform: Transform, entity: Entity) -> Transform {
        let buffer = self
            .data
            .entry(transform.t_type)
            .or_insert_with(|| Vec::new());
        let i = buffer.len();
        buffer.resize(i + 8, 0.0); // 8 floats per transform
        transform.buffer_index = i / 8;
        let entity_vec = self
            .entity_map
            .entry(transform.t_type)
            .or_insert_with(|| Vec::new());
        entity_vec.push(entity);
        self.update(transform);
        transform
    }

    // Swaps the transform to delete with the last transform in the buffer and truncates the buffer
    pub fn delete(&mut self, transform: Transform) -> Option<DeleteInfo> // Return the swapped transform
    {
        let i = transform.buffer_index * 8;
        let buffer = self.data.get_mut(&transform.t_type).unwrap_or_else(|| {
            godot_print!(
                "TransformBuffer: ThingType {:?} not found",
                transform.t_type
            );
            panic!(
                "wrong delete call: ThingType {:?} not found",
                transform.t_type
            );
        });
        let last_index = buffer.len() - 8;
        let last_slice = &buffer[last_index..last_index + 8].to_vec();
        if i != last_index {
            buffer[i..i + 8].copy_from_slice(last_slice);
        }
        buffer.truncate(last_index);
        let entity_vec = self
            .entity_map
            .get_mut(&transform.t_type)
            .unwrap_or_else(|| {
                godot_print!(
                    "TransformBuffer: ThingType {:?} not found",
                    transform.t_type
                );
                panic!(
                    "wrong delete call: ThingType {:?} not found",
                    transform.t_type
                );
            });
        if i != last_index {
            let swapped_entity = entity_vec.pop().unwrap();
            entity_vec[transform.buffer_index] = swapped_entity;
            return Some(DeleteInfo {
                swapped_entity,
                swapped_index: transform.buffer_index,
            });
        } else {
            entity_vec.pop(); // Remove the last entity
            return None; // Return None if no entity was swapped
        }
    }

    pub fn update(&mut self, transform: Transform) {
        if let Some(buffer) = self.data.get_mut(&transform.t_type) {
            let (sin, cos) = transform.rotation.sin_cos();
            let i = transform.buffer_index * 8;
            buffer[i] = cos * transform.scale.x; // x.x
            buffer[i + 1] = sin * transform.scale.y; // y.x
            buffer[i + 2] = 0.0; // padding
            buffer[i + 3] = transform.position.x; // x.w (translation x)
            buffer[i + 4] = sin * transform.scale.x; // x.y
            buffer[i + 5] = -cos * transform.scale.y; // y.y
            buffer[i + 6] = 0.0; // padding
            buffer[i + 7] = transform.position.y; // y.w (translation y)
        }
    }

    pub fn get_buffer(&self, t_type: ThingType) -> Option<&Vec<f32>> {
        self.data.get(&t_type)
    }
}
