use std::collections::{HashMap, HashSet};

use crate::ecs::prelude::*;
use bevy_ecs::{prelude::*, system::IntoResult};
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
        }
    }
    fn ready(&mut self) {
        self.world.insert_resource(TransformBuffer::new());
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
        self.world.add_observer(despawn_order_trigger);

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
        for i in 0..20 {
            self.world.trigger(SpawnUnitEvent {
                transform: Transform {
                    position: Vector2::new(
                        (i % 10) as f32 * 40.0 + 50.0,
                        (i / 10) as f32 * 40.0 + 50.0,
                    ),
                    rotation: 0.0,
                    scale: Vector2::new(1.0, 1.0),
                    size: 10.0,
                    buffer_index: 0,
                    t_type: ThingType::Test,
                },
                stats: UnitMovement {
                    speed: 0.0,
                    max_speed: 100.0,
                    acceleration: 200.0,
                    moving: false,
                    dir_vec: Vector2::ZERO,
                    preferred_dir: Vector2::ZERO,
                    dist_target_sq: f32::MAX,
                },
            });
        }

        self.schedule.add_systems(transform_update_system);
        self.schedule.add_systems(despawn_units_system);
        self.schedule.add_systems(update_flow_field_system);
        self.schedule.add_systems(update_spatial_grid_system);
        self.schedule.add_systems(acceleration_system);
        self.schedule.add_systems(delayed_stop_system);
        self.schedule.add_systems(
            (
                flow_movement_system,
                avoid_system,
                smooth_wall_passing_system,
                apply_move_system,
            )
                .chain(),
        );
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
        let transform_buffer = self.world.resource_mut::<TransformBuffer>();
        let Some(buffer) = transform_buffer.get_buffer(t_type) else {
            godot_error!("TransformBuffer: ThingType {:?} not found", t_type);
            return;
        };
        if (buffer.len() / 8) != (multimesh.get_instance_count() as usize) {
            multimesh.set_instance_count((buffer.len() / 8) as i32);
        }
        let buffer = PackedFloat32Array::from(buffer[..buffer.len()].as_ref());

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
            .query_filtered::<Entity, (With<Transform>, With<UnitMovement>, With<Selected>)>()
            .iter(&self.world)
            .collect::<HashSet<_>>();
        self.world.trigger(MoveOrderEvent {
            target_position: target,
            units,
        });
    }

    #[func]
    pub fn get_flow_vectors(&mut self) -> PackedFloat32Array {
        let flow_grid = self.world.resource::<FlowGrid>();
        let (width, height) = (flow_grid.width, flow_grid.height);
        let mut buffer = vec![0.0; width * height * 4]; // 각 셀마다 (world_x, world_y, flow_x, flow_y) 4개의 float
        let cell_size = flow_grid.cell_size;
        let flow_field = self.world.query::<(&FlowField,)>().iter(&self.world).next();
        if let Some((flow_field,)) = flow_field {
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let world_x = (x as f32 + 0.5) * cell_size;
                    let world_y = (y as f32 + 0.5) * cell_size;
                    if idx * 4 + 3 >= buffer.len() || idx >= flow_field.field.len() {
                        continue; // 버퍼 범위를 벗어나지 않도록 체크
                    }
                    buffer[idx * 4] = world_x;
                    buffer[idx * 4 + 1] = world_y;
                    if let Some(vec) = &flow_field.field[idx] {
                        buffer[idx * 4 + 2] = vec.x;
                        buffer[idx * 4 + 3] = vec.y;
                    }
                }
            }
        }
        PackedFloat32Array::from(buffer.as_slice())
    }

    #[func]
    pub fn select_unit_in_area(&mut self, top_left: Vector2, bottom_right: Vector2) {
        let sgrid = self.world.resource::<SpatialGrid>();
        if let Ok(result_vec) = sgrid.query_entities_rect(top_left, bottom_right) {
            for entity_info in result_vec {
                self.world.entity_mut(entity_info.entity).insert(Selected);
            }
        } else {
            godot_error!(
                "SpatialGrid query failed in area {:?} to {:?}",
                top_left,
                bottom_right
            );
            return;
        }
    }

    #[func]
    pub fn remove_selection(&mut self) {
        let selected_entities = self
            .world
            .query_filtered::<Entity, With<Selected>>()
            .iter(&self.world)
            .collect::<Vec<_>>();
        for entity in selected_entities {
            self.world.entity_mut(entity).remove::<Selected>();
        }
    }

    #[func]
    pub fn get_selected_units(&mut self) -> PackedInt32Array {
        let selected_units = self
            .world
            .query_filtered::<&Transform, (With<UnitMovement>, With<Selected>)>()
            .iter(&self.world)
            .flat_map(|t| [t.t_type as i32, t.buffer_index as i32])
            .collect::<Vec<_>>();
        PackedInt32Array::from(selected_units.as_slice())
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
