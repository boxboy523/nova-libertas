use std::collections::HashMap;

use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::{
    classes::{MultiMesh, ProjectSettings},
    prelude::*,
};

const CELL_SIZE: f32 = 40.0;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct UnitManager {
    world: World,
    schedule: Schedule,
    base: Base<Node>,
    render_buffer: HashMap<ThingType, Vec<f32>>,
    #[export]
    map_csv_path: GString,
}

#[godot_api]
impl INode for UnitManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::default(),
            base,
            map_csv_path: GString::new(),
            render_buffer: HashMap::new(),
        }
    }
    fn ready(&mut self) {
        let transform_buffer = TransformBuffer::new(64);
        self.world.insert_resource(transform_buffer);
        let path = ProjectSettings::singleton().globalize_path(&self.map_csv_path);
        let (map_width, map_height, wall_data) = load_map_from_csv(&path.to_string());
        let map_width_f = map_width as f32 * CELL_SIZE;
        let map_height_f = map_height as f32 * CELL_SIZE;
        let flow_grid = FlowGrid::new(map_width_f, map_height_f, CELL_SIZE, &wall_data);
        self.world.insert_resource(flow_grid);
        let spatial_grid = SpatialGrid::new(map_width_f, map_height_f, CELL_SIZE, &wall_data);
        self.world.insert_resource(spatial_grid);
        let time = Time { delta: 0.0 };
        self.world.insert_resource(time);

        self.world.add_observer(spawn_units_trigger);
        self.world.add_observer(move_order_trigger);
        self.world.add_observer(spawn_wall_trigger);

        for y in 0..map_height {
            for x in 0..map_width {
                if wall_data[y * map_width + x] {
                    self.world.trigger(SpawnWallEvent {
                        position: Vector2::new(x as f32 * CELL_SIZE, y as f32 * CELL_SIZE),
                        size: Vector2::new(CELL_SIZE, CELL_SIZE),
                    });
                }
            }
        }

        // 테스트용 유닛 2,000개 일괄 생성 (가로 50줄, 세로 40줄)
        for i in 0..1 {
            self.world.trigger(SpawnUnitEvent {
                transform: Transform {
                    position: Vector2::new(
                        (i % 50) as f32 * 40.0 + 50.0,
                        (i / 50) as f32 * 40.0 + 50.0,
                    ),
                    rotation: 0.0,
                    scale: Vector2::new(1.0, 1.0),
                    size: 10.0,
                },
                stats: UnitMovement {
                    speed: 0.0,
                    max_speed: 100.0,
                    acceleration: 20.0,
                    moving: false,
                    dir_vec: Vector2::ZERO,
                    seperation_force: Vector2::ZERO,
                },
                t_type: ThingType::Test,
            });
        }

        self.schedule.add_systems(apply_move_system);
        self.schedule.add_systems(flow_movement_system);
        self.schedule.add_systems(seperation_force_system);
        self.schedule.add_systems(transform_update_system);
        self.schedule.add_systems(despawn_units_system);
        self.schedule.add_systems(update_flow_field_system);
        self.schedule.add_systems(update_spatial_grid_system);
        self.schedule.add_systems(acceleration_system);
    }

    fn physics_process(&mut self, delta: f64) {
        self.world.resource_mut::<Time>().delta = delta as f32;
        self.schedule.run(&mut self.world);
    }
}

#[godot_api]
impl UnitManager {
    #[func]
    pub fn update_multimesh_buffer(&mut self, thing_type: i64, mut multimesh: Gd<MultiMesh>) {
        let t_type = match thing_type {
            0 => ThingType::Test,
            1 => ThingType::Wall,
            _ => return, // 지원하지 않는 ThingType이면 함수 종료
        };
        let mut transform_buffer = self.world.resource_mut::<TransformBuffer>();
        let buffer = &mut self.render_buffer.entry(t_type).or_insert_with(Vec::new);
        let mut buffer_len = 0;
        for chunk_id in 0..transform_buffer.chunks.len() {
            if transform_buffer.chunks[chunk_id].t_type != Some(t_type) {
                continue; // 현재 ThingType과 일치하는 청크가 아니면 건너뜀
            }
            //if !transform_buffer.chunks[chunk_id].modified {
            //    buffer_idx += 1;
            //    continue;
            //}
            let chunk_len = transform_buffer.chunks[chunk_id].length;
            set_or_append(
                buffer,
                buffer_len,
                &transform_buffer.data
                    [chunk_id * CHUNK_SIZE * 8..chunk_id * CHUNK_SIZE * 8 + chunk_len * 8],
            );
            transform_buffer.chunks[chunk_id].modified = false;
            buffer_len += chunk_len * 8;
        }
        if (buffer_len / 8) != (multimesh.get_instance_count() as usize) {
            multimesh.set_instance_count((buffer_len / 8) as i32);
        }

        //godot_print!("Buffer length for {:?}: {}", t_type, buffer_len / 8);

        let buffer = PackedFloat32Array::from(buffer[..buffer_len].as_ref());

        // 고도 엔진의 렌더링 서버에 메모리 블록 통째로 덮어쓰기
        multimesh.set_buffer(&buffer);
    }

    #[func]
    pub fn get_unit_count(&self) -> i32 {
        (self.world.resource::<TransformBuffer>().data.len() / 8) as i32
    }

    #[func]
    pub fn order_move(&mut self, target: Vector2) {
        let units = self
            .world
            .query_filtered::<Entity, (With<Transform>, With<UnitMovement>)>()
            .iter(&self.world)
            .collect::<Vec<_>>();
        self.world.trigger(MoveOrderEvent {
            target_position: target,
            units,
        });
    }
}

fn set_or_append(buffer: &mut Vec<f32>, index: usize, value: &[f32]) {
    if index + value.len() <= buffer.len() {
        buffer[index..index + value.len()].copy_from_slice(value);
    } else {
        buffer.resize(index, 0.0);
        buffer.extend_from_slice(value);
    }
}

pub fn load_map_from_csv(path: &str) -> (usize, usize, Vec<bool>) {
    let content = std::fs::read_to_string(path).unwrap();
    let mut wall = Vec::new();
    let mut width = 0;
    let mut height = 0;
    for line in content.lines() {
        height += 1;
        let row: Vec<bool> = line.split(',').map(|c| c.trim() == "1").collect();
        width = row.len();
        wall.extend(row);
    }
    (width, height, wall)
}
