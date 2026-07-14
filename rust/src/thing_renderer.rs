use crate::{ecs::prelude::*, unit_manager::UnitManager};
use godot::{
    classes::{IMultiMeshInstance2D, MultiMeshInstance2D},
    prelude::*,
};

#[derive(GodotClass, Debug)]
#[class(base=MultiMeshInstance2D)]
pub struct ThingRenderer {
    base: Base<MultiMeshInstance2D>,
    #[export]
    t_type: ThingType,
    #[export]
    unit_manager: Option<Gd<UnitManager>>,
}

#[godot_api]
impl IMultiMeshInstance2D for ThingRenderer {
    fn init(base: Base<MultiMeshInstance2D>) -> Self {
        Self {
            base,
            t_type: ThingType::Test,
            unit_manager: None,
        }
    }

    fn physics_process(&mut self, _: f64) {
        godot_print!(
            "ThingRenderer: physics_process called for {:?}",
            self.t_type
        );
        let unit_manager = match &self.unit_manager {
            Some(manager) => manager.bind(),
            None => return,
        };
        godot_print!("ThingRenderer: unit_manager is {:?}", unit_manager);
        let Some(buf) = unit_manager.get_transform_buf(self.t_type) else {
            return;
        };
        let Some(mut multimesh) = self.base().get_multimesh() else {
            return;
        };
        if (buf.len() / 8) != multimesh.get_instance_count() as usize {
            multimesh.set_instance_count((buf.len() / 8) as i32);
        }
        multimesh.set_buffer(&PackedFloat32Array::from(buf));
    }
}
