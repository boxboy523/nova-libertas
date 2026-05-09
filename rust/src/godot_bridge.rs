use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::{classes::MultiMesh, prelude::*};

const MAP_WIDTH: f32 = 1000.0;
const MAP_HEIGHT: f32 = 1000.0;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct UnitManager {
    world: World,
    schedule: Schedule,
    base: Base<Node>,
}

#[godot_api]
impl INode for UnitManager {
    fn init(base: Base<Node>) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        let transform_buffer = TransformBuffer::new(2048);
        world.insert_resource(transform_buffer);
        let flow_grid = FlowGrid::new(MAP_WIDTH, MAP_HEIGHT, 20.0);
        world.insert_resource(flow_grid);
        let spatial_grid = SpatialGrid::new(MAP_WIDTH, MAP_HEIGHT, 20.0);
        world.insert_resource(spatial_grid);
        let time = Time { delta: 0.0 };
        world.insert_resource(time);

        world.add_observer(spawn_units_trigger);
        world.add_observer(move_order_trigger);
        // 테스트용 유닛 2,000개 일괄 생성 (가로 50줄, 세로 40줄)
        for i in 0..100 {
            world.trigger(SpawnUnitEvent {
                transform: Transform {
                    position: Vector2::new((i % 50) as f32 * 20.0, (i / 50) as f32 * 20.0),
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
                },
            });
        }

        schedule.add_systems(apply_move_system);
        schedule.add_systems(flow_movement_system);
        schedule.add_systems(seperation_force_system);
        schedule.add_systems(transform_update_system);
        schedule.add_systems(despawn_units_system);
        schedule.add_systems(update_flow_field_system);
        schedule.add_systems(update_spatial_grid_system);
        schedule.add_systems(acceleration_system);

        Self {
            world,
            schedule,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // Rust ECS 내부 시뮬레이션 틱 진행
        // (원한다면 delta 값을 Bevy의 Resource로 넣어 시스템에서 읽게 할 수 있습니다)
        self.world.resource_mut::<Time>().delta = delta as f32;
        self.schedule.run(&mut self.world);
    }
}

// 4. Zero-copy 렌더링 브릿지 함수
#[godot_api]
impl UnitManager {
    #[func]
    pub fn update_multimesh_buffer(&mut self, mut multimesh: Gd<MultiMesh>) {
        let buffer = self.world.resource_mut::<TransformBuffer>();
        if (buffer.data.len() / 8) != (multimesh.get_instance_count() as usize) {
            multimesh.set_instance_count((buffer.data.len() / 8) as i32);
        }

        let buffer = PackedFloat32Array::from(buffer.data.as_slice());

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
            .query_filtered::<Entity, With<Transform>>()
            .iter(&self.world)
            .collect::<Vec<_>>();
        self.world.trigger(MoveOrderEvent {
            target_position: target,
            units,
        });
    }
}
