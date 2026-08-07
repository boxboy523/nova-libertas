use std::collections::HashMap;

use crate::prelude::*;
use bevy::prelude::*;

pub mod data;
pub mod info;
pub mod system;

#[derive(Debug, Clone)]
pub struct UnitVisual {
    pub kind: UnitVisualKind,
    pub size: Vec2,
}

impl UnitVisual {
    pub fn get_mesh_mat(
        &self,
        kind: Option<AnimationKind>,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        match &self.kind {
            UnitVisualKind::Simple { mesh, material, .. } => (mesh.clone(), material.clone()),
            UnitVisualKind::AnimationSet(animation_set) => {
                let data = animation_set.get_data(kind.unwrap_or(AnimationKind::Stand));
                (data.frame_meshes[0].normal.clone(), data.material.clone())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnitVisualKind {
    Simple {
        material: Handle<StandardMaterial>,
        mesh: Handle<Mesh>,
    },
    AnimationSet(AnimationSet),
}

#[derive(Resource, Debug, Default)]
pub struct SpriteCatalog {
    pub sprites: HashMap<ThingType, UnitVisual>,
}

#[derive(Debug, Clone)]
pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteCatalog>()
            .add_systems(Startup, system::sprite_catalog_startup_system)
            .add_systems(
                Update,
                (
                    system::change_animation_system,
                    system::look_dir_system,
                    system::animation_system,
                )
                    .chain(),
            );
    }
}
