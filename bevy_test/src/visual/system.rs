use crate::prelude::*;
use bevy::prelude::*;
use strum::IntoEnumIterator;

pub fn sprite_catalog_startup_system(
    mut catalog: ResMut<SpriteCatalog>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                let material = materials.add(StandardMaterial {
                    base_color_texture: Some(image),
                    unlit: true,
                    alpha_mode: AlphaMode::Mask(0.1),
                    cull_mode: None,
                    ..default()
                });
                let mesh = meshes.add(Mesh::from(Rectangle::from_size(
                    sprite_conf.sprite_info.size,
                )));
                UnitVisualKind::Simple { material, mesh }
            }

            SpriteInfoKind::AnimationSet(anim_info) => {
                if anim_info.stand.frame_count > anim_info.stand.columns {
                    panic!(
                        "Stand animation frame count exceeds \
                         the number of cells in the sprite sheet"
                    );
                }

                let image = asset_server
                    .load(info_asset_path.join(anim_info.stand.file.unwrap_or("stand.png".into())));
                let material = materials.add(StandardMaterial {
                    base_color_texture: Some(image),
                    unlit: true,
                    alpha_mode: AlphaMode::Mask(0.1),
                    cull_mode: None,
                    ..default()
                });

                let mut mesh_vec = Vec::new();

                for row in 0..anim_info.stand.rows {
                    for col in 0..anim_info.stand.columns {
                        let normal = meshes.add(create_atlas_quad(
                            sprite_conf.sprite_info.size,
                            anim_info.stand.columns,
                            anim_info.stand.rows,
                            col,
                            row,
                            false,
                            sprite_conf.sprite_info.offset,
                        ));
                        let flipped = meshes.add(create_atlas_quad(
                            sprite_conf.sprite_info.size,
                            anim_info.stand.columns,
                            anim_info.stand.rows,
                            col,
                            row,
                            true,
                            sprite_conf.sprite_info.offset,
                        ));
                        mesh_vec.push(AnimationFrameMesh { normal, flipped });
                    }
                }

                let stand_clip = AnimationData {
                    material,
                    frame_meshes: mesh_vec,
                    columns: anim_info.stand.columns,
                    rows: anim_info.stand.rows,
                    frame_count: anim_info.stand.frame_count,
                    fps: anim_info.stand.fps,
                    looping: anim_info.stand.looping,
                };

                let moving_clip = anim_info.moving.map(|moving_info| {
                    if moving_info.frame_count > moving_info.columns {
                        panic!(
                            "Moving animation frame count exceeds \
                             the number of cells in the sprite sheet"
                        );
                    }

                    let image = asset_server
                        .load(info_asset_path.join(moving_info.file.unwrap_or("move.png".into())));

                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(image),
                        unlit: true,
                        alpha_mode: AlphaMode::Mask(0.1),
                        cull_mode: None,
                        ..default()
                    });

                    let mut mesh_vec = Vec::new();

                    for row in 0..moving_info.rows {
                        for col in 0..moving_info.columns {
                            let normal = meshes.add(create_atlas_quad(
                                sprite_conf.sprite_info.size,
                                moving_info.columns,
                                moving_info.rows,
                                col,
                                row,
                                false,
                                sprite_conf.sprite_info.offset,
                            ));
                            let flipped = meshes.add(create_atlas_quad(
                                sprite_conf.sprite_info.size,
                                moving_info.columns,
                                moving_info.rows,
                                col,
                                row,
                                true,
                                sprite_conf.sprite_info.offset,
                            ));
                            mesh_vec.push(AnimationFrameMesh { normal, flipped });
                        }
                    }

                    AnimationData {
                        material,
                        frame_meshes: mesh_vec,
                        columns: moving_info.columns,
                        rows: moving_info.rows,
                        frame_count: moving_info.frame_count,
                        fps: moving_info.fps,
                        looping: moving_info.looping,
                    }
                });

                let attacking_clip = anim_info.attacking.map(|attacking_info| {
                    if attacking_info.frame_count > attacking_info.columns {
                        panic!(
                            "Attacking animation frame count exceeds \
                                 the number of cells in the sprite sheet"
                        );
                    }

                    let image = asset_server.load(
                        info_asset_path.join(attacking_info.file.unwrap_or("attack.png".into())),
                    );

                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(image),
                        unlit: true,
                        alpha_mode: AlphaMode::Mask(0.1),
                        cull_mode: None,
                        ..default()
                    });

                    let mut mesh_vec = Vec::new();

                    for row in 0..attacking_info.rows {
                        for col in 0..attacking_info.columns {
                            let normal = meshes.add(create_atlas_quad(
                                sprite_conf.sprite_info.size,
                                attacking_info.columns,
                                attacking_info.rows,
                                col,
                                row,
                                false,
                                sprite_conf.sprite_info.offset,
                            ));
                            let flipped = meshes.add(create_atlas_quad(
                                sprite_conf.sprite_info.size,
                                attacking_info.columns,
                                attacking_info.rows,
                                col,
                                row,
                                true,
                                sprite_conf.sprite_info.offset,
                            ));
                            mesh_vec.push(AnimationFrameMesh { normal, flipped });
                        }
                    }

                    AnimationData {
                        material,
                        frame_meshes: mesh_vec,
                        columns: attacking_info.columns,
                        rows: attacking_info.rows,
                        frame_count: attacking_info.frame_count,
                        fps: attacking_info.fps,
                        looping: attacking_info.looping,
                    }
                });

                UnitVisualKind::AnimationSet(AnimationSet {
                    stand: stand_clip,
                    moving: moving_clip,
                    attacking: attacking_clip,
                })
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
            let world_dir = Vec3::new(movement.dir_vec.x, 0.0, movement.dir_vec.y).normalize();
            let camera_right = camera.right().as_vec3();
            let camera_up = camera.up().as_vec3();
            let screen_dir = Vec2::new(world_dir.dot(camera_right), world_dir.dot(camera_up));
            let (row, flip) = dir_to_row(screen_dir);
            anim_state.fliped = flip;
            anim_state.dir_idx = row;
            if let Some(unit_visual) = catalog.sprites.get(t_type) {
                if let UnitVisualKind::AnimationSet(animation_set) = &unit_visual.kind {
                    if let Some(cur_anim) = cur_anim_opt {
                        let data = animation_set.get_data(cur_anim.0);
                        *mesh = Mesh3d(if anim_state.fliped {
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

                let index = (anim_state.columns * anim_state.dir_idx + anim_state.frame) as usize;
                if let Some(unit_sprite) = catalog.sprites.get(t_type) {
                    if let UnitVisualKind::AnimationSet(animation_set) = &unit_sprite.kind {
                        if let Some(cur_anim) = cur_anim_opt {
                            let data = animation_set.get_data(cur_anim.0);
                            *mesh = Mesh3d(if anim_state.fliped {
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
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Changed<CurrentAnimation>,
    >,
    catalog: Res<SpriteCatalog>,
) {
    query
        .iter_mut()
        .for_each(|(entity, t_type, current_anim, mut mesh, mut material)| {
            if let Some(unit_visual) = catalog.sprites.get(t_type) {
                if let UnitVisualKind::AnimationSet(animation_set) = &unit_visual.kind {
                    let data = animation_set.get_data(current_anim.0);
                    material.0 = data.material.clone();
                    mesh.0 = data.frame_meshes[0].normal.clone();
                    commands
                        .entity(entity)
                        .insert(AnimationState::from_data(data));
                }
            }
        });
}

fn dir_to_row(dir: Vec2) -> (u32, bool) {
    let angle = dir.to_angle();
    let oct = (angle / (std::f32::consts::PI / 4.0))
        .round()
        .rem_euclid(8.0) as u32;

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
