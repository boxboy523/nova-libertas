use std::collections::HashMap;

use crate::prelude::*;
use bevy::prelude::*;

pub mod data;
pub mod info;
pub mod system;
pub mod team_color;

#[derive(Debug, Clone)]
pub struct UnitVisual {
    pub kind: UnitVisualKind,
    pub size: Vec2,
    pub anchor: Option<VisualAnchor>,
    pub roll_offset: f32,
}

impl UnitVisual {
    pub fn get_mesh_mat(
        &self,
        kind: Option<AnimationKind>,
        team: Option<Team>,
    ) -> (Handle<Mesh>, Handle<TeamColorMaterial>) {
        match &self.kind {
            UnitVisualKind::Simple { mesh, material, .. } => (
                mesh.clone(),
                material
                    .get(&team.unwrap_or(Team::Neutral))
                    .expect("Material not found for team")
                    .clone(),
            ),
            UnitVisualKind::AnimationSet(animation_set) => {
                let Some(data) = animation_set
                    .animations
                    .get(&kind.unwrap_or(AnimationKind::Stand))
                else {
                    panic!("Animation data not found for kind: {:?}", kind);
                };
                (
                    data.frame_meshes[0].normal.clone(),
                    data.material
                        .get(&team.unwrap_or(Team::Neutral))
                        .expect("Material not found for team")
                        .clone(),
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnitVisualKind {
    Simple {
        material: HashMap<Team, Handle<TeamColorMaterial>>,
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
                    system::update_cur_anim_system,
                )
                    .chain(),
            );
    }
}
