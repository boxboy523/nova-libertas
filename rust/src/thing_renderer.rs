use crate::{ecs::prelude::*, unit_manager::UnitManager};
use godot::{
    classes::{
        multi_mesh::TransformFormat, IMultiMeshInstance2D, Mesh, MultiMesh, MultiMeshInstance2D,
    },
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
    #[export]
    mesh: Option<Gd<Mesh>>,
    #[export]
    sheet_size: Vector2,
    #[export]
    y_sorted: bool,
}

#[godot_api]
impl IMultiMeshInstance2D for ThingRenderer {
    fn init(base: Base<MultiMeshInstance2D>) -> Self {
        Self {
            base,
            t_type: ThingType::Test,
            unit_manager: None,
            mesh: None,
            sheet_size: Vector2::new(1.0, 1.0),
            y_sorted: false,
        }
    }

    fn ready(&mut self) {
        let multimesh = multimesh_gen(self.mesh.clone());
        self.base_mut().set_multimesh(&multimesh);
    }

    fn physics_process(&mut self, _: f64) {
        let unit_manager = if let Some(manager) = &self.unit_manager {
            manager.bind()
        } else {
            return;
        };
        let Some(buf) = unit_manager.get_transform_buf(self.t_type, self.y_sorted) else {
            return;
        };
        let Some(mut multimesh) = self.base().get_multimesh() else {
            return;
        };
        if (buf.len() / STRIDE) != multimesh.get_instance_count() as usize {
            multimesh.set_instance_count((buf.len() / STRIDE) as i32);
        }
        multimesh.set_buffer(&buf);
    }
}

fn multimesh_gen(mesh: Option<Gd<Mesh>>) -> Gd<MultiMesh> {
    let mut mm = MultiMesh::new_gd();
    mm.set_transform_format(TransformFormat::TRANSFORM_2D);
    mm.set_use_colors(false);
    mm.set_use_custom_data(true);
    if let Some(mesh) = mesh {
        mm.set_mesh(&mesh);
    } else {
        godot_error!("No mesh provided for ThingRenderer, using default QuadMesh");
    }
    mm.set_instance_count(0);
    mm
}
