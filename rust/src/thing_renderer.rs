use crate::{ecs::prelude::*, unit_manager::UnitManager};
use godot::{
    classes::{
        multi_mesh::TransformFormat, IMultiMeshInstance2D, Mesh, MultiMesh, MultiMeshInstance2D,
        QuadMesh, ShaderMaterial, Texture2D,
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
    #[export]
    hp_texture: Option<Gd<Texture2D>>,
    #[export]
    hp_shader: Option<Gd<ShaderMaterial>>,
    hp_renderer: Option<Gd<MultiMeshInstance2D>>,
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
            hp_texture: None,
            hp_shader: None,
            hp_renderer: None,
        }
    }

    fn ready(&mut self) {
        let multimesh = multimesh_gen(self.mesh.clone());
        self.base_mut().set_multimesh(&multimesh);
        if let Some(hp_texture) = &self.hp_texture {
            let mut hp_renderer = MultiMeshInstance2D::new_alloc();
            let mut hp_mesh = QuadMesh::new_gd();
            hp_mesh.set_size(
                self.t_type
                    .hp_bar_style()
                    .map_or(Vector2::new(1.0, 1.0), |style| style.size),
            );
            hp_renderer.set_multimesh(&multimesh_gen(Some(hp_mesh.upcast())));
            hp_renderer.set_texture(hp_texture);
            hp_renderer.set_material(self.hp_shader.as_ref());
            let renderer_node = hp_renderer.clone().upcast::<Node>();
            self.hp_renderer = Some(hp_renderer);
            self.base_mut().add_child(&renderer_node);
        }
    }

    fn physics_process(&mut self, _: f64) {
        let unit_manager = if let Some(manager) = &self.unit_manager {
            manager.bind()
        } else {
            return;
        };
        let (opt_buf, opt_hp_buf) = unit_manager.get_transform_buf(self.t_type, self.y_sorted);
        let Some(buf) = opt_buf else {
            return;
        };
        let Some(mut multimesh) = self.base().get_multimesh() else {
            return;
        };
        if (buf.len() / STRIDE) != multimesh.get_instance_count() as usize {
            multimesh.set_instance_count((buf.len() / STRIDE) as i32);
        }
        multimesh.set_buffer(&buf);
        if let Some(hp_renderer) = &self.hp_renderer {
            let Some(mut hp_multimesh) = hp_renderer.get_multimesh() else {
                return;
            };
            let Some(hp_buf) = opt_hp_buf else {
                return;
            };
            if (hp_buf.len() / STRIDE) != hp_multimesh.get_instance_count() as usize {
                hp_multimesh.set_instance_count((hp_buf.len() / STRIDE) as i32);
            }
            hp_multimesh.set_buffer(&hp_buf);
        }
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
