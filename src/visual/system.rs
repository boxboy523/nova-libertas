use crate::{
    combat::component::Attack,
    constants::SWITCH_FACE_ANGLE,
    movement::component::{Moving, Stopped, UnitMovement},
    thing::{ThingCatalog, ThingId},
    unit::component::Team,
    visual::{SpriteCatalog, UnitVisual, UnitVisualKind, data::{AnimationData, AnimationFrameMesh, AnimationKind, AnimationSet, AnimationState, CurrentAnimation}, event::ChangeVisualEvent, info::{SpriteConfig, SpriteInfoKind}, team_color::TeamColorMaterial},
    world3d::{Billboard, create_atlas_quad, create_simple_quad},
};
use bevy::prelude::*;

pub fn sprite_catalog_startup_system(
    thing_catalog: Res<ThingCatalog>,
    mut catalog: ResMut<SpriteCatalog>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TeamColorMaterial>>,
) {
    thing_catalog.iter().for_each(|(id, entry)| {
        let info_path = thing_catalog.asset_root.join(&entry.asset_dir);
        let info_asset_path = entry.asset_dir.clone();
        let text = std::fs::read_to_string(info_path.join("info.toml"))
            .expect("Failed to read thing info file");

        let sprite_conf: SpriteConfig =
            toml::from_str(&text).expect("UnitSprite: Failed to parse thing info file");

        let world_size = sprite_conf.sprite_info.world_size(&info_path)
            .unwrap_or_else(|error| panic!("Invalid sprite configuration for {id:?}: {error:#}"));
        let visual_kind = match &sprite_conf.sprite_info.kind {
            SpriteInfoKind::Simple { file } => {
                let image = asset_server.load(info_asset_path.join(file));
                let material = TeamColorMaterial::get_team_hashmap(&image, &mut materials);
                let mesh = meshes.add(create_simple_quad(
                    world_size,
                    sprite_conf.sprite_info.sprite_offset,
                ));
                UnitVisualKind::Simple { mesh, material }
            }

            SpriteInfoKind::AnimationSet { animations } => {
                if !animations.contains_key(&AnimationKind::Stand) {
                    panic!("AnimationSet must contain a 'stand' animation for {:?}", id);
                }
                UnitVisualKind::AnimationSet(AnimationSet{ animations: animations.iter().map(|(kind, anim_info)| {
                    let frame_size = sprite_conf.sprite_info.scaled_size(anim_info.cell_size)
                        .unwrap_or_else(|error| panic!("Invalid {kind:?} sprite size for {id:?}: {error}"));
                    if anim_info.frame_count > anim_info.columns {
                        panic!(
                            "Animation frame count exceeds the number of cells in the sprite sheet for {:?}",
                            kind
                        );
                    }

                    let image = asset_server.load(info_asset_path.join(anim_info.file.clone().unwrap_or(kind.default_file())));

                    let material = TeamColorMaterial::get_team_hashmap(&image, &mut materials);

                    let mut mesh_vec = Vec::new();

                    for row in 0..anim_info.rows {
                        for col in 0..anim_info.columns {
                            let normal = meshes.add(create_atlas_quad(
                                frame_size,
                                anim_info.columns,
                                anim_info.rows,
                                col,
                                row,
                                false,
                                sprite_conf.sprite_info.sprite_offset,
                            ));
                            let flipped = meshes.add(create_atlas_quad(
                                frame_size,
                                anim_info.columns,
                                anim_info.rows,
                                col,
                                row,
                                true,
                                sprite_conf.sprite_info.sprite_offset,
                            ));
                            mesh_vec.push(AnimationFrameMesh { normal, flipped });
                        }
                    }

                    (kind.clone(), AnimationData {
                        material,
                        frame_meshes: mesh_vec,
                        columns: anim_info.columns,
                        rows: anim_info.rows,
                        frame_count: anim_info.frame_count,
                        fps: anim_info.fps,
                        looping: anim_info.looping,
                    })
                }).collect()})
            }
        };

        catalog.sprites.insert(
            *id,
            UnitVisual {
                kind: visual_kind,
                size: world_size,
                model_offset: sprite_conf.sprite_info.model_offset,
                anchor: sprite_conf.sprite_info.anchor,
                roll_offset: sprite_conf.sprite_info.roll_offset_degrees.to_radians(),
            },
        );
    });
}

pub fn look_dir_system(
    mut commands: Commands,
    parent_query: Query<(&UnitMovement, &ThingId, Option<&Team>), Changed<UnitMovement>>,
    query: Query<(&ChildOf, &AnimationState, Entity)>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
) {
    query.iter().for_each(
        |(parent, anim_state, entity)| {
            let Ok((movement, id, opt_team)) = parent_query.get(parent.0) else {
                return;
            };
            if movement.preferred_dir.length_squared() < f32::EPSILON {
                return;
            }

            let world_dir =
                Vec3::new(movement.preferred_dir.x, 0.0, movement.preferred_dir.y).normalize();
            let camera_right = camera.right().as_vec3();
            let camera_up = camera.up().as_vec3();
            let screen_dir = Vec2::new(world_dir.dot(camera_right), world_dir.dot(camera_up));
            let angle = screen_dir.to_angle();
            let current_center = anim_state.facing_oct as f32 * (std::f32::consts::FRAC_PI_4);
            let delta = (angle - current_center + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            if delta.abs() < SWITCH_FACE_ANGLE {
                return;
            }
            let facing_oct = (angle / std::f32::consts::FRAC_PI_4)
                .round()
                .rem_euclid(8.0) as u32;
            commands.trigger(ChangeVisualEvent {
                visual_entity: entity,
                facing_oct: Some(facing_oct),
                id: *id,
                team: opt_team.copied().unwrap_or(Team::Neutral),
                animation_kind: None,
            });
        },
    );
}

pub fn animation_system(
    time: Res<Time>,
    query_parent: Query<&ThingId>,
    mut query: Query<(&ChildOf, &mut AnimationState, &mut Mesh3d)>,
    catalog: Res<SpriteCatalog>,
) {
    query
        .iter_mut()
        .for_each(|(parent, mut anim_state, mut mesh)| {
            anim_state.tick(time.delta_secs());
            if anim_state.mesh_dirty {
                let t_type = query_parent.get(parent.0).expect("Parent entity does not have ThingType");
                let Some(frame_mesh) = catalog.get_data(*t_type, anim_state.kind)
                    .and_then(|data| {
                        let (row, flip) = oct_to_row(anim_state.facing_oct);
                        let index = (data.columns * row + anim_state.frame) as usize;
                        if flip {
                            data.frame_meshes.get(index).map(|frame| frame.flipped.clone())
                        } else {
                            data.frame_meshes.get(index).map(|frame| frame.normal.clone())
                        }
                    }) else {
                        warn!("Animation data for ThingType {:?} and AnimationKind {:?} not found in catalog", t_type, anim_state.kind);
                        return;
                    };
                *mesh = Mesh3d(frame_mesh);
                anim_state.mesh_dirty = false;
            }
        });
}

pub fn change_animation_system(
    mut commands: Commands,
    query_parent: Query<(&CurrentAnimation, &ThingId, Option<&Team>), Changed<CurrentAnimation>>,
    query: Query<(&ChildOf, Entity), With<Billboard>>,
) {
    query.iter().for_each(
        |(parent, entity)| {
            if let Ok((cur_anim, id, opt_team)) = query_parent.get(parent.0) {
                commands.trigger(ChangeVisualEvent {
                    visual_entity: entity,
                    facing_oct: None,
                    id: *id,
                    team: opt_team.copied().unwrap_or(Team::Neutral),
                    animation_kind: Some(cur_anim.0),
                });
            }
        },
    );
}

pub fn update_cur_anim_system(
    mut query_moving: Query<
        (Option<&Attack>, &mut CurrentAnimation),
        (Or<(Added<Moving>, Changed<Attack>)>, Without<Stopped>),
    >,
    mut query_stopped: Query<
        (&Stopped, &mut CurrentAnimation),
        (Changed<Stopped>, Without<Moving>),
    >,
) {
    query_moving
        .iter_mut()
        .for_each(|(attack_opt, mut cur_anim)| {
            if let Some(attack) = attack_opt {
                if attack.attacking && cur_anim.0 != AnimationKind::Attack {
                    *cur_anim = CurrentAnimation(AnimationKind::Attack);
                }
                return;
            }
            if cur_anim.0 != AnimationKind::Move {
                *cur_anim = CurrentAnimation(AnimationKind::Move);
            }
        });

    query_stopped
        .iter_mut()
        .for_each(|(stopped, mut cur_anim)| {
            if stopped.in_range && cur_anim.0 != AnimationKind::Stand {
                *cur_anim = CurrentAnimation(AnimationKind::Stand);
            } else if !stopped.in_range && cur_anim.0 != AnimationKind::Move {
                *cur_anim = CurrentAnimation(AnimationKind::Move);
            }
        });
}

fn oct_to_row(oct: u32) -> (u32, bool) {
    match oct {
        0 => (2, true),  // Right
        1 => (3, true),  // Up-Right
        2 => (4, false), // Up
        3 => (3, false), // Up-Left
        4 => (2, false), // Left
        5 => (1, false), // Down-Left
        6 => (0, false), // Down
        7 => (1, true),  // Down-Right
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn simple_sprite_offset_translates_vertices_without_scaling_or_uv_changes() {
        let size = Vec2::new(76.8, 60.0);
        let offset = Vec2::new(5.0, -20.0);
        let original = create_simple_quad(size, Vec2::ZERO);
        let shifted = create_simple_quad(size, offset);
        let Some(VertexAttributeValues::Float32x3(original_positions)) =
            original.attribute(Mesh::ATTRIBUTE_POSITION)
        else { panic!("Expected position vertices") };
        let Some(VertexAttributeValues::Float32x3(shifted_positions)) =
            shifted.attribute(Mesh::ATTRIBUTE_POSITION)
        else { panic!("Expected position vertices") };
        for (original, shifted) in original_positions.iter().zip(shifted_positions) {
            assert!((shifted[0] - original[0] - offset.x).abs() < 0.0001);
            assert!((shifted[1] - original[1] - offset.y).abs() < 0.0001);
            assert_eq!(shifted[2], original[2]);
        }
        let Some(VertexAttributeValues::Float32x2(original_uvs)) =
            original.attribute(Mesh::ATTRIBUTE_UV_0)
        else { panic!("Expected UV vertices") };
        let Some(VertexAttributeValues::Float32x2(shifted_uvs)) =
            shifted.attribute(Mesh::ATTRIBUTE_UV_0)
        else { panic!("Expected UV vertices") };
        assert_eq!(original_uvs, shifted_uvs);
    }

    #[test]
    fn simple_sprite_zero_offset_anchors_bottom_at_origin() {
        let mesh = create_simple_quad(Vec2::new(40.0, 60.0), Vec2::ZERO);
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else { panic!("Expected position vertices") };
        for position in positions {
            assert_eq!(position[0].abs(), 20.0);
            assert_eq!(position[2], 0.0);
        }
        let ys = positions.iter().map(|position| position[1]);
        assert_eq!(ys.clone().reduce(f32::min), Some(0.0));
        assert_eq!(ys.reduce(f32::max), Some(60.0));
    }
}
