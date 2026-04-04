use crate::ecs::prelude::*;
use bevy_ecs::prelude::*;
use godot::{classes::MultiMesh, prelude::*};

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
        let time = Time { delta: 0.0 };
        world.insert_resource(time);

        world.add_observer(spawn_units_trigger);
        // 테스트용 유닛 2,000개 일괄 생성 (가로 50줄, 세로 40줄)
        for i in 0..2000 {
            world.trigger(SpawnUnitEvent {
                transform: Transform {
                    position: Vector2::new((i % 50) as f32 * 20.0, (i / 50) as f32 * 20.0),
                    rotation: 0.0,
                    scale: Vector2::new(1.0, 1.0),
                },
                stats: UnitStats { speed: 50.0 },
            });
        }

        schedule.add_systems(movement_system);
        schedule.add_systems(transform_update_system);
        schedule.add_systems(despawn_units_system);

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
}
