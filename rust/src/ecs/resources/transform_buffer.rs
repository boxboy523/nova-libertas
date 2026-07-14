use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::prelude::*;
use std::collections::HashMap;
use strum::EnumIter;

pub const STRIDE: usize = 12; // 8 floats per transform (2x4 matrix)

#[derive(GodotConvert, Var, Export, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Default)]
#[godot(via = i64)]
pub enum ThingType {
    #[default]
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

    pub fn add(
        &mut self,
        mut transform: Transform,
        dir: Option<Vector2>,
        team: Option<Team>,
        entity: Entity,
    ) -> Transform {
        let buffer = self
            .data
            .entry(transform.t_type)
            .or_insert_with(|| Vec::new());
        let i = buffer.len();
        buffer.resize(i + STRIDE, 0.0); // 8 floats per transform
        transform.buffer_index = i / STRIDE;
        let entity_vec = self
            .entity_map
            .entry(transform.t_type)
            .or_insert_with(|| Vec::new());
        entity_vec.push(entity);
        self.update(transform, dir, team);
        transform
    }

    // Swaps the transform to delete with the last transform in the buffer and truncates the buffer
    pub fn delete(&mut self, transform: Transform) -> Option<DeleteInfo> // Return the swapped transform
    {
        let i = transform.buffer_index * STRIDE;
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
        let last_index = buffer.len() - STRIDE;
        let last_slice = &buffer[last_index..last_index + STRIDE].to_vec();
        if i != last_index {
            buffer[i..i + STRIDE].copy_from_slice(last_slice);
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

    pub fn update(&mut self, transform: Transform, dir: Option<Vector2>, team: Option<Team>) {
        if let Some(buffer) = self.data.get_mut(&transform.t_type) {
            let (sin, cos) = transform.rotation.sin_cos();
            let (row, flip) = if let Some(dir) = dir {
                dir_to_row(dir)
            } else {
                (0, false)
            };
            let scale_x = if flip {
                -transform.scale.x
            } else {
                transform.scale.x
            };
            let i = transform.buffer_index * STRIDE;
            buffer[i] = cos * scale_x; // x.x
            buffer[i + 1] = sin * transform.scale.y; // y.x
            buffer[i + 2] = 0.0; // padding
            buffer[i + 3] = transform.position.x; // x.w (translation x)
            buffer[i + 4] = sin * scale_x; // x.y
            buffer[i + 5] = -cos * transform.scale.y; // y.y
            buffer[i + 6] = 0.0; // padding
            buffer[i + 7] = transform.position.y; // y.w (translation y)
            buffer[i + 8] = team.map_or(0.0, |t| t as i32 as f32); // custom_data.x (team)
            buffer[i + 9] = row as f32; // custom_data.y (row)
        }
    }

    pub fn get_buffer(&self, t_type: ThingType) -> Option<&Vec<f32>> {
        self.data.get(&t_type)
    }
}

fn dir_to_row(dir: Vector2) -> (u32, bool) {
    let angle = dir.angle();
    let oct = (angle / (std::f32::consts::PI / 4.0))
        .round()
        .rem_euclid(8.0) as u32;

    match oct {
        0 => (2, false), // Right
        1 => (1, false), // Up-Right
        2 => (0, false), // Up
        3 => (1, true),  // Up-Left
        4 => (2, true),  // Left
        5 => (3, true),  // Down-Left
        6 => (4, false), // Down
        7 => (3, false), // Down-Right
        _ => unreachable!(),
    }
}
