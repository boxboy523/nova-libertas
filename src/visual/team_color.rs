use std::collections::HashMap;

use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};
use strum::IntoEnumIterator;

use crate::prelude::Team;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct TeamColorMaterial {
    #[uniform(0)]
    pub team_color: LinearRgba,

    #[uniform(0)]
    pub key_hue: f32,
    #[uniform(0)]
    pub tolerance: f32,
    #[uniform(0)]
    pub min_saturation: f32,
    #[uniform(0)]
    pub alpha_cutoff: f32,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Material for TeamColorMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/team_color.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(self.alpha_cutoff)
    }
}

impl TeamColorMaterial {
    pub fn get_team_hashmap(
        image: &Handle<Image>,
        materials: &mut ResMut<Assets<TeamColorMaterial>>,
    ) -> HashMap<Team, Handle<TeamColorMaterial>> {
        let mut map = HashMap::new();
        Team::iter().for_each(|team| {
            let material = materials.add(TeamColorMaterial {
                team_color: team.color().into(),
                key_hue: 0.33,
                tolerance: 0.05,
                min_saturation: if team == Team::Neutral { 2.0 } else { 0.2 }, // Disable color replacement for neutral team by setting min_saturation > 1.0
                alpha_cutoff: 0.1,
                texture: Some(image.clone()),
            });
            map.insert(team, material);
        });
        map
    }
}
