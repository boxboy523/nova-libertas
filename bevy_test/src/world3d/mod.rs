use crate::{
    map::{GameMap, TerrainHeightMap},
    prelude::*,
};
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
            .add_systems(Update, (rts_camera_system, billboard_system));
    }
}

fn setup_world_3d(
    mut commands: Commands,
    mut height_map: ResMut<TerrainHeightMap>,
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
        RtsCamera {
            focus: Vec3::new(500.0, 0.0, 500.0),
            radius: 1500.0,
        },
        Transform::from_xyz(500.0, 700.0, 1100.0).looking_at(center, Vec3::Y),
    ));

    height_map.spawn_map(&mut commands, &mut materials, &mut meshes);

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
pub struct RtsCamera {
    pub focus: Vec3,
    pub radius: f32,
}

fn rts_camera_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(&mut Transform, &mut RtsCamera)>,
    window: Single<&Window>,
) {
    let (mut transform, mut rts_cam) = camera.into_inner();
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let delta = time.delta_secs();

    let cam_speed = 1.2;
    let zoom_speed = 500.0;

    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();

    let ground_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let ground_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::ArrowLeft) || cursor.x <= CAMERA_MOUSE_DEADZONE {
        direction -= ground_right;
    }

    if keys.pressed(KeyCode::ArrowRight) || cursor.x >= window.width() - CAMERA_MOUSE_DEADZONE {
        direction += ground_right;
    }

    if keys.pressed(KeyCode::ArrowUp) || cursor.y <= CAMERA_MOUSE_DEADZONE {
        direction += ground_forward;
    }

    if keys.pressed(KeyCode::ArrowDown) || cursor.y >= window.height() - CAMERA_MOUSE_DEADZONE {
        direction -= ground_forward;
    }

    if keys.pressed(KeyCode::KeyQ) {
        rts_cam.radius -= zoom_speed * delta;
    }

    if keys.pressed(KeyCode::KeyE) {
        rts_cam.radius += zoom_speed * delta;
    }

    rts_cam.radius = rts_cam.radius.clamp(200.0, 2000.0);

    let radius = rts_cam.radius;
    rts_cam.focus += direction * cam_speed * delta * radius;

    let horizontal = CAMERA_PITCH.cos();

    let offset = Vec3::new(
        CAMERA_YAW.sin() * horizontal,
        CAMERA_PITCH.sin(),
        CAMERA_YAW.cos() * horizontal,
    ) * rts_cam.radius;

    transform.translation = rts_cam.focus + offset;
    transform.look_at(rts_cam.focus, Vec3::Y);
}
