use std::collections::{HashMap, HashSet};

use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::{classes::ProjectSettings, prelude::*};
use strum::IntoEnumIterator;

const CELL_SIZE: f32 = 40.0;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct UnitManager {
    world: World,
    schedule: Schedule,
    base: Base<Node2D>,
    #[export]
    map_csv_path: GString,
}

#[godot_api]
impl INode2D for UnitManager {
    fn init(base: Base<Node2D>) -> Self {
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
        self.world.add_observer(attack_order_trigger);
        self.world.add_observer(damage_trigger);

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
        for i in 0..30 {
            let position =
                Vector2::new((i % 10) as f32 * 40.0 + 50.0, (i / 10) as f32 * 40.0 + 50.0);
            self.world.trigger(SpawnUnitEvent {
                transform: Transform {
                    position,
                    scale: Vector2::new(1.0, 1.0),
                    size: 15.0,
                    t_type: ThingType::Test,
                    ..Default::default()
                },
                team: if i % 2 == 0 {
                    Team::Player
                } else {
                    Team::Enemy
                },
                hp: 100.0,
            });
        }

        self.schedule.add_systems(transform_update_system);
        self.schedule.add_systems(despawn_units_system);
        self.schedule.add_systems(update_flow_field_system);
        self.schedule.add_systems(update_spatial_grid_system);
        self.schedule.add_systems(acceleration_system);
        self.schedule.add_systems(delayed_stop_system);
        self.schedule.add_systems(stop_attacking_unit_system);
        self.schedule.add_systems(stop_moving_unit_system);
        self.schedule.add_systems(remove_empty_orders_system);
        self.schedule.add_systems(flow_field_added_system);
        self.schedule.add_systems(
            (
                flow_movement_system,
                stopped_in_range_system,
                auto_attack_system,
                move_or_attack_system,
                attack_system,
                avoid_system,
                smooth_wall_passing_system,
                apply_move_system,
            )
                .chain(),
        );
        for t_type in ThingType::iter() {
            godot_print!("UnitManager: Emitting update_type for {:?}", t_type);
            self.base_mut()
                .call_deferred("update_type", &[(t_type as i64).to_variant()]);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        self.world.resource_mut::<Time>().delta = delta as f32;
        self.schedule.run(&mut self.world);
        self.base_mut().queue_redraw();
    }

    fn draw(&mut self) {
        let select_draw_call = self
            .world
            .query_filtered::<&Transform, (With<UnitMovement>, With<Selected>)>()
            .iter(&self.world)
            .map(|t| (t.position, t.size))
            .collect::<Vec<_>>();
        for (pos, size) in select_draw_call {
            self.base_mut()
                .draw_circle(pos, size + 5.0, Color::from_rgba(0.0, 1.0, 0.0, 1.0));
        }
    }
}

#[godot_api]
impl UnitManager {
    #[signal]
    pub fn selection_changed(thing_type: i64, indices: PackedInt32Array);

    #[signal]
    pub fn t_type_changed(thing_type: i64, indices: PackedInt32Array, team: PackedInt32Array);

    pub fn get_transform_buf(
        &self,
        t_type: ThingType,
        y_sorted: bool,
    ) -> (Option<PackedFloat32Array>, Option<PackedFloat32Array>) {
        let transform_buffer = self.world.resource::<TransformBuffer>();
        let buf = transform_buffer.get_buffer(t_type);
        if y_sorted {
            if let Some(buf) = buf {
                let n = buf.objects.len() / STRIDE;
                let mut order: Vec<usize> = (0..n).collect();
                order.sort_unstable_by(|&i, &j| {
                    let y_i = buf.objects[i * STRIDE + 7];
                    let y_j = buf.objects[j * STRIDE + 7];
                    y_i.partial_cmp(&y_j).unwrap_or(std::cmp::Ordering::Equal)
                });
                let mut sorted_buf = Vec::with_capacity(buf.objects.len());
                for &i in &order {
                    sorted_buf.extend_from_slice(&buf.objects[i * STRIDE..(i + 1) * STRIDE]);
                }
                let sorted_buf_hp = if let Some(buf_hp) = buf.hp_bars.as_ref() {
                    let mut sorted_buf_hp = Vec::with_capacity(buf_hp.len());
                    for &i in &order {
                        sorted_buf_hp.extend_from_slice(&buf_hp[i * STRIDE..(i + 1) * STRIDE]);
                    }
                    Some(PackedFloat32Array::from(sorted_buf_hp.as_slice()))
                } else {
                    None
                };
                (
                    Some(PackedFloat32Array::from(sorted_buf.as_slice())),
                    sorted_buf_hp,
                )
            } else {
                (None, None)
            }
        } else {
            (
                buf.map(|b| PackedFloat32Array::from(b.objects.as_slice())),
                buf.map(|b| {
                    b.hp_bars
                        .as_ref()
                        .map(|hp| PackedFloat32Array::from(hp.as_slice()))
                })
                .flatten(),
            )
        }
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
            auto_attack: false,
        });
    }

    #[func]
    pub fn order_move_with_auto_attack(&mut self, target: Vector2) {
        let units = self
            .world
            .query_filtered::<Entity, (With<Transform>, With<UnitMovement>, With<Selected>)>()
            .iter(&self.world)
            .collect::<HashSet<_>>();
        self.world.trigger(MoveOrderEvent {
            target_position: target,
            units,
            auto_attack: true,
        });
    }

    #[func]
    pub fn order_attack(&mut self, target_pos: Vector2) {
        let units = self
            .world
            .query_filtered::<Entity, (With<Transform>, With<UnitMovement>, With<Selected>)>()
            .iter(&self.world)
            .collect::<HashSet<_>>();
        let spatial_grid = self.world.resource::<SpatialGrid>();
        let mut target_entities = match spatial_grid.query_entities(target_pos, 1.0, false) {
            Ok(entities) => entities,
            Err(_) => {
                godot_error!("SpatialGrid query failed at position {:?}", target_pos);
                return;
            }
        };
        target_entities.sort_unstable_by(|a, b| {
            (target_pos - a.pos)
                .length_squared()
                .total_cmp(&(target_pos - b.pos).length_squared())
        });
        if let Some(target_entity) = target_entities.first() {
            self.world.trigger(AttackOrderEvent {
                target: target_entity.entity,
                units,
            });
        } else {
            godot_warn!("No target entity found at position {:?}", target_pos);
        }
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
        let units = if top_left == bottom_right {
            if let Some(units) = sgrid.get_entities_at(top_left) {
                Ok(units)
            } else {
                Ok(vec![])
            }
        } else {
            sgrid.query_entities_rect(top_left, bottom_right)
        };
        if let Ok(result_vec) = units {
            let mut q = self.world.query::<&Team>();
            let to_select = result_vec
                .into_iter()
                .filter(|e| {
                    if let Ok(team) = q.get(&self.world, e.entity) {
                        *team == Team::Player
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            for entity in to_select {
                self.world.entity_mut(entity.entity).insert(Selected);
            }
        } else {
            godot_error!(
                "SpatialGrid query failed in area {:?} to {:?}",
                top_left,
                bottom_right
            );
            return;
        }

        let mut grouped = HashMap::new();
        let mut q = self.world.query_filtered::<&Transform, With<Selected>>();
        q.iter(&self.world).for_each(|transform| {
            grouped
                .entry(transform.t_type)
                .or_insert_with(Vec::new)
                .push(transform.buffer_index as i32);
        });

        for (t_type, indices) in grouped {
            self.base_mut().call_deferred(
                "emit_signal",
                &[
                    "selection_changed".to_variant(),
                    (t_type as i64).to_variant(),
                    PackedInt32Array::from(indices.as_slice()).to_variant(),
                ],
            );
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

        for t in ThingType::iter() {
            self.base_mut().emit_signal(
                "selection_changed",
                &[
                    (t as i64).to_variant(),
                    PackedInt32Array::new().to_variant(),
                ],
            );
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

    #[func]
    pub fn update_type(&mut self, t_type: ThingType) {
        let mut indices = Vec::new();
        let mut teams = Vec::new();
        let mut q = self.world.query::<(&Transform, &Team)>();
        q.iter(&self.world).for_each(|(transform, team)| {
            if transform.t_type == t_type {
                indices.push(transform.buffer_index as i32);
                teams.push(*team as i32);
            }
        });
        self.base_mut().emit_signal(
            "t_type_changed",
            &[
                (t_type as i32).to_variant(),
                PackedInt32Array::from(indices.as_slice()).to_variant(),
                PackedInt32Array::from(teams.as_slice()).to_variant(),
            ],
        );
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
