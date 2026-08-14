use crate::prelude::*;
use bevy::prelude::*;
use strum::IntoEnumIterator;

pub fn sprite_catalog_startup_system(
    mut catalog: ResMut<SpriteCatalog>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TeamColorMaterial>>,
) {
    ThingType::iter().for_each(|t_type| {
        let info_path = t_type.get_path();
        let info_asset_path = t_type.get_assets_path();
        let text = std::fs::read_to_string(info_path.join("info.toml"))
            .expect("Failed to read thing info file");

        let sprite_conf: SpriteConfig =
            toml::from_str(&text).expect("UnitSprite: Failed to parse thing info file");

        let visual_kind = match sprite_conf.sprite_info.kind {
            SpriteInfoKind::Simple { file } => {
                let image = asset_server.load(info_asset_path.join(file));
                let material = TeamColorMaterial::get_team_hashmap(&image, &mut materials);
                let mesh = meshes.add(Mesh::from(Rectangle::from_size(
                    sprite_conf.sprite_info.size,
                )));
                UnitVisualKind::Simple { mesh, material }
            }

            SpriteInfoKind::AnimationSet { animations } => {
                if !animations.contains_key(&AnimationKind::Stand) {
                    panic!("AnimationSet must contain a 'stand' animation for {:?}", t_type);
                }
                UnitVisualKind::AnimationSet(AnimationSet{ animations: animations.iter().map(|(kind, anim_info)| {
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
                                sprite_conf.sprite_info.size,
                                anim_info.columns,
                                anim_info.rows,
                                col,
                                row,
                                false,
                                sprite_conf.sprite_info.offset,
                            ));
                            let flipped = meshes.add(create_atlas_quad(
                                sprite_conf.sprite_info.size,
                                anim_info.columns,
                                anim_info.rows,
                                col,
                                row,
                                true,
                                sprite_conf.sprite_info.offset,
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
            t_type,
            UnitVisual {
                kind: visual_kind,
                size: sprite_conf.sprite_info.size,
            },
        );
    });
}

pub fn look_dir_system(
    mut query: Query<(
        &UnitMovement,
        &mut AnimationState,
        &mut Mesh3d,
        &ThingType,
        Option<&CurrentAnimation>,
    )>,
    catalog: Res<SpriteCatalog>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
) {
    query.iter_mut().for_each(
        |(movement, mut anim_state, mut mesh, t_type, cur_anim_opt)| {
            if movement.preferred_dir.length_squared() < f32::EPSILON {
                return;
            }
            if movement.speed < 1.0 {
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
            anim_state.facing_oct = (angle / std::f32::consts::FRAC_PI_4)
                .round()
                .rem_euclid(8.0) as u32;
            let (row, flip) = oct_to_row(anim_state.facing_oct);
            if let Some(unit_visual) = catalog.sprites.get(t_type) {
                if let UnitVisualKind::AnimationSet(animation_set) = &unit_visual.kind {
                    if let Some(cur_anim) = cur_anim_opt {
                        let data = animation_set
                            .get(&cur_anim.0)
                            .unwrap_or(animation_set.get(&AnimationKind::Stand).unwrap());
                        *mesh = Mesh3d(if flip {
                            data.frame_meshes
                                [(anim_state.columns * row + anim_state.frame) as usize]
                                .flipped
                                .clone()
                        } else {
                            data.frame_meshes
                                [(anim_state.columns * row + anim_state.frame) as usize]
                                .normal
                                .clone()
                        });
                    }
                }
            }
        },
    );
}

pub fn animation_system(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationState,
        &mut Mesh3d,
        &ThingType,
        Option<&CurrentAnimation>,
    )>,
    catalog: Res<SpriteCatalog>,
) {
    query
        .iter_mut()
        .for_each(|(mut anim_state, mut mesh, t_type, cur_anim_opt)| {
            anim_state.timer.tick(time.delta());
            if anim_state.timer.just_finished() {
                anim_state.frame += 1;
                if anim_state.frame >= anim_state.frame_count {
                    if anim_state.looping {
                        anim_state.frame = 0;
                    } else {
                        anim_state.frame = anim_state.frame_count - 1;
                    }
                }

                let (row, flip) = oct_to_row(anim_state.facing_oct);
                let index = (anim_state.columns * row + anim_state.frame) as usize;
                if let Some(unit_sprite) = catalog.sprites.get(t_type) {
                    if let UnitVisualKind::AnimationSet(animation_set) = &unit_sprite.kind {
                        if let Some(cur_anim) = cur_anim_opt {
                            let data = animation_set
                                .get(&cur_anim.0)
                                .unwrap_or(animation_set.get(&AnimationKind::Stand).unwrap());
                            *mesh = Mesh3d(if flip {
                                data.frame_meshes[index].flipped.clone()
                            } else {
                                data.frame_meshes[index].normal.clone()
                            });
                        }
                    }
                }
            }
        });
}

pub fn change_animation_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ThingType,
            &CurrentAnimation,
            Option<&AnimationState>,
            &mut Mesh3d,
            &mut MeshMaterial3d<TeamColorMaterial>,
            Option<&Team>,
        ),
        Changed<CurrentAnimation>,
    >,
    catalog: Res<SpriteCatalog>,
) {
    query.iter_mut().for_each(
        |(entity, t_type, current_anim, opt_anim_state, mut mesh, mut material, team)| {
            if let Some(unit_visual) = catalog.sprites.get(t_type) {
                if let UnitVisualKind::AnimationSet(animation_set) = &unit_visual.kind {
                    let data = animation_set
                        .get(&current_anim.0)
                        .unwrap_or(animation_set.get(&AnimationKind::Stand).unwrap());
                    material.0 = data
                        .material
                        .get(&team.unwrap_or(&Team::Neutral))
                        .expect("Material not found for team")
                        .clone();

                    let mut new_anim_state = AnimationState::from_data(data);
                    if let Some(old_state) = opt_anim_state {
                        new_anim_state.facing_oct = old_state.facing_oct;
                        let (row, flip) = oct_to_row(new_anim_state.facing_oct);
                        mesh.0 = if flip {
                            data.frame_meshes[(data.columns * row) as usize]
                                .flipped
                                .clone()
                        } else {
                            data.frame_meshes[(data.columns * row) as usize]
                                .normal
                                .clone()
                        };
                    } else {
                        mesh.0 = data.frame_meshes[0].normal.clone();
                    }
                    commands.entity(entity).insert(new_anim_state);
                }
            }
        },
    );
}

pub fn update_cur_anim_system(
    mut query_moving: Query<
        (&mut CurrentAnimation, Option<&Attack>),
        (Added<Moving>, Without<Stopped>),
    >,
    mut query_stopped: Query<
        (&mut CurrentAnimation, &Stopped),
        (Changed<Stopped>, Without<Moving>),
    >,
) {
    query_moving
        .iter_mut()
        .for_each(|(mut cur_anim, attack_opt)| {
            if let Some(attack) = attack_opt {
                if attack.attacking {
                    *cur_anim = CurrentAnimation(AnimationKind::Attack);
                }
                return;
            }
            *cur_anim = CurrentAnimation(AnimationKind::Move);
        });

    query_stopped
        .iter_mut()
        .for_each(|(mut cur_anim, stopped)| {
            if stopped.in_range {
                *cur_anim = CurrentAnimation(AnimationKind::Stand);
            } else {
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
