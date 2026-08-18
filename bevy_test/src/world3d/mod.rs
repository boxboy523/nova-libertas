use crate::prelude::*;
use bevy::{mesh::VertexAttributeValues, prelude::*};

pub struct World3DPlugin;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Billboard {
    pub roll: f32,
    roll_offset: f32,
}

impl Plugin for World3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world_3d)
            .add_systems(Update, (orbit_camera_system, billboard_system));
    }
}

fn setup_world_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let center = Vec3::new(500.0, 0.0, 500.0);
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        OrbitCamera {
            focus: Vec3::new(500.0, 0.0, 500.0),
            radius: 900.0,
            yaw: 0.0,
            pitch: 45.0_f32.to_radians(),
        },
        Transform::from_xyz(500.0, 700.0, 1100.0).looking_at(center, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1000.0, 1000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(500.0, 0.0, 500.0),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_4,
            0.0,
        )),
    ));
}

pub fn spawn_billboard(
    commands: &mut Commands,
    visual: &UnitVisual,
    target: Entity,
    team: Option<Team>,
) {
    let (mesh, material) = visual.get_mesh_mat(None, team);
    commands.entity(target).insert((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Billboard {
            roll: 0.0,
            roll_offset: visual.roll_offset,
        },
    ));
}

fn billboard_system(
    mut query: Query<(&mut Transform, &Billboard)>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
) {
    let camera_rotation = camera.rotation();
    query.iter_mut().for_each(|(mut transform, billboard)| {
        transform.rotation =
            camera_rotation * Quat::from_rotation_z(billboard.roll + billboard.roll_offset);
    });
}

pub fn create_atlas_quad(
    size: Vec2,
    columns: u32,
    rows: u32,
    column: u32,
    row: u32,
    fliped: bool,
    offset: Vec2,
) -> Mesh {
    let mut mesh = Mesh::from(Rectangle::new(size.x, size.y));

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for position in positions {
            position[0] += offset.x;
            position[1] += size.y * 0.5 + offset.y;
        }
    }

    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    else {
        return mesh;
    };

    for uv in uvs {
        let local_u = if fliped { 1.0 - uv[0] } else { uv[0] };
        uv[0] = (column as f32 + local_u) / columns as f32;
        uv[1] = (uv[1] + row as f32) / rows as f32;
    }
    mesh
}

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
}

fn orbit_camera_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(&mut Transform, &mut OrbitCamera)>,
) {
    let (mut transform, mut orbit) = camera.into_inner();
    let delta = time.delta_secs();

    let rotation_speed = 1.2;
    let zoom_speed = 500.0;

    if keys.pressed(KeyCode::ArrowLeft) {
        orbit.yaw += rotation_speed * delta;
    }

    if keys.pressed(KeyCode::ArrowRight) {
        orbit.yaw -= rotation_speed * delta;
    }

    if keys.pressed(KeyCode::ArrowUp) {
        orbit.pitch += rotation_speed * delta;
    }

    if keys.pressed(KeyCode::ArrowDown) {
        orbit.pitch -= rotation_speed * delta;
    }

    if keys.pressed(KeyCode::KeyQ) {
        orbit.radius -= zoom_speed * delta;
    }

    if keys.pressed(KeyCode::KeyE) {
        orbit.radius += zoom_speed * delta;
    }

    orbit.pitch = orbit
        .pitch
        .clamp(10.0_f32.to_radians(), 80.0_f32.to_radians());

    orbit.radius = orbit.radius.clamp(200.0, 2000.0);

    let horizontal = orbit.pitch.cos();

    let offset = Vec3::new(
        orbit.yaw.sin() * horizontal,
        orbit.pitch.sin(),
        orbit.yaw.cos() * horizontal,
    ) * orbit.radius;

    transform.translation = orbit.focus + offset;
    transform.look_at(orbit.focus, Vec3::Y);
}
