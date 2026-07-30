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

#[derive(Debug, Clone, Copy)]
pub struct HpBarStyle {
    pub offset: Vector2,
    pub size: Vector2,
}

impl ThingType {
    pub fn hp_bar_style(&self) -> Option<HpBarStyle> {
        match self {
            ThingType::Test => Some(HpBarStyle {
                offset: Vector2::new(0.0, -20.0),
                size: Vector2::new(40.0, 5.0),
            }),
            ThingType::Wall => None,
        }
    }

    pub fn get_unitstats(&self) -> Option<UnitStats> {
        match self {
            ThingType::Test => Some(UnitStats {
                max_speed: 100.0,
                acceleration: 200.0,
                max_hp: 100.0,
            }),
            ThingType::Wall => None,
        }
    }

    pub fn get_unit_battle_stats(&self) -> Option<UnitBattleStats> {
        match self {
            ThingType::Test => Some(UnitBattleStats {
                attack_range: 100.0,
                attack_damage: 10.0,
                attack_cooldown: 1.0,
            }),
            ThingType::Wall => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BufferData {
    pub objects: Vec<f32>, // 8 floats per transform (2x4 matrix) + 4 floats for custom data
    pub entities: Vec<Entity>, // buffer_index -> Entity
    pub hp_bars: Option<Vec<f32>>,
}

impl BufferData {
    pub fn add(
        &mut self,
        mut transform: Transform,
        dir: Option<Vector2>,
        team: Option<Team>,
        entity: Entity,
        hp_ratio: Option<f32>,
    ) -> Transform {
        let i = self.objects.len();
        self.objects.resize(i + STRIDE, 0.0); // 8 floats per transform
        if hp_ratio.is_some() {
            if self.hp_bars.is_none() {
                self.hp_bars = Some(Vec::new());
            }
            let hp_buffer = self.hp_bars.as_mut().unwrap();
            let hp_i = hp_buffer.len();
            hp_buffer.resize(hp_i + STRIDE, 0.0); // 8 floats per transform
        }
        transform.buffer_index = i / STRIDE;
        self.entities.push(entity);
        self.update(transform, dir, team);
        if let Some(hp_ratio) = hp_ratio {
            self.update_hp(transform, hp_ratio);
        }
        transform
    }

    pub fn update(&mut self, transform: Transform, dir: Option<Vector2>, team: Option<Team>) {
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
        self.objects[i] = cos * scale_x; // x.x
        self.objects[i + 1] = sin * transform.scale.y; // y.x
        self.objects[i + 2] = 0.0; // padding
        self.objects[i + 3] = transform.position.x; // x.w (translation x)
        self.objects[i + 4] = sin * scale_x; // x.y
        self.objects[i + 5] = -cos * transform.scale.y; // y.y
        self.objects[i + 6] = 0.0; // padding
        self.objects[i + 7] = transform.position.y; // y.w (translation y)
        self.objects[i + 8] = team.map_or(0.0, |t| t as i32 as f32); // custom_data.x (team)
        self.objects[i + 9] = row as f32; // custom_data.y (row)
    }

    pub fn update_hp(&mut self, transform: Transform, hp_ratio: f32) {
        let hp_buffer = self.hp_bars.as_mut().unwrap();
        let Some(style) = transform.t_type.hp_bar_style() else {
            return;
        };
        let hp_transform_pos = transform.position + style.offset;
        let i = transform.buffer_index * STRIDE;
        hp_buffer[i] = 1.0; // x.x (scale x)
        hp_buffer[i + 1] = 0.0;
        hp_buffer[i + 2] = 0.0;
        hp_buffer[i + 3] = hp_transform_pos.x; // x.w (translation x)
        hp_buffer[i + 4] = 0.0;
        hp_buffer[i + 5] = -1.0; // y.y (scale y)
        hp_buffer[i + 6] = 0.0;
        hp_buffer[i + 7] = hp_transform_pos.y; // y.w (translation y
        hp_buffer[i + 8] = hp_ratio; // custom_data.x (hp ratio)
    }

    pub fn delete(&mut self, buffer_index: usize) -> Option<Entity> {
        let i = buffer_index * STRIDE;
        let last_index = self.objects.len() - STRIDE;
        let last_slice = &self.objects[last_index..last_index + STRIDE].to_vec();
        if i != last_index {
            self.objects[i..i + STRIDE].copy_from_slice(last_slice);
        }
        self.objects.truncate(last_index);
        if let Some(hp_buffer) = self.hp_bars.as_mut() {
            let last_hp_index = hp_buffer.len() - STRIDE;
            let last_hp_slice = &hp_buffer[last_hp_index..last_hp_index + STRIDE].to_vec();
            if i != last_index {
                hp_buffer[i..i + STRIDE].copy_from_slice(last_hp_slice);
            }
            hp_buffer.truncate(last_hp_index);
        }
        let swapped_entity = if i != last_index {
            let swapped_entity = self.entities.pop().unwrap();
            self.entities[buffer_index] = swapped_entity;
            Some(swapped_entity)
        } else {
            self.entities.pop();
            None
        };
        swapped_entity
    }
}

// ECS 시스템에서 유닛의 위치, 회전, 크기 등의 변환 정보를 관리하는 버퍼
#[derive(Resource)]
pub struct TransformBuffer {
    pub data: HashMap<ThingType, BufferData>, // ThingType -> BufferData
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
        transform: Transform,
        dir: Option<Vector2>,
        team: Option<Team>,
        entity: Entity,
        hp_ratio: Option<f32>,
    ) -> Transform {
        self.data
            .entry(transform.t_type)
            .or_insert_with(|| BufferData::default())
            .add(transform, dir, team, entity, hp_ratio)
    }

    // Swaps the transform to delete with the last transform in the buffer and truncates the buffer
    pub fn delete(&mut self, transform: Transform) -> Option<DeleteInfo> // Return the swapped transform
    {
        if let Some(buffer) = self.data.get_mut(&transform.t_type) {
            let swapped_entity = buffer.delete(transform.buffer_index)?;
            Some(DeleteInfo {
                swapped_entity,
                swapped_index: transform.buffer_index,
            })
        } else {
            None
        }
    }

    pub fn update(&mut self, transform: Transform, dir: Option<Vector2>, team: Option<Team>) {
        self.data
            .get_mut(&transform.t_type)
            .map(|buffer| buffer.update(transform, dir, team));
    }

    pub fn update_hp(&mut self, transform: Transform, hp_ratio: f32) {
        self.data
            .get_mut(&transform.t_type)
            .map(|buffer| buffer.update_hp(transform, hp_ratio));
    }

    pub fn get_buffer(&self, t_type: ThingType) -> Option<&BufferData> {
        self.data.get(&t_type)
    }
}

fn dir_to_row(dir: Vector2) -> (u32, bool) {
    let angle = dir.angle();
    let oct = (angle / (std::f32::consts::PI / 4.0))
        .round()
        .rem_euclid(8.0) as u32;

    match oct {
        0 => (2, true),  // Right
        1 => (1, true),  // Down-Right
        2 => (0, false), // Down
        3 => (1, false), // Down-Left
        4 => (2, false), // Left
        5 => (3, false), // Up-Left
        6 => (4, false), // Up
        7 => (3, true),  // Up-Right
        _ => unreachable!(),
    }
}
