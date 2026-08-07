use crate::prelude::*;
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_draw);
    }
}

fn debug_draw(
    mut gizmos: Gizmos,
    query_units: Query<(Entity, &Position, &UnitStats)>,
    query_selected: Query<&Selected>,
    mouse_state: Res<MouseState>,
) {
    query_units.iter().for_each(|(entity, position, stats)| {
        let pos = Vec3::new(position.x, 0.0, position.y);
        let radius = stats.size;
        let iso = Isometry3d {
            translation: pos.into(),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        };
        gizmos.circle(iso, radius, Color::srgb(0.0, 1.0, 0.0));
        if query_selected.get(entity).is_ok() {
            gizmos.circle(iso, radius + 10.0, Color::srgb(1.0, 1.0, 0.0));
        }
    });
    let mouse_pos = Vec3::new(
        mouse_state.world_position.x,
        0.1,
        mouse_state.world_position.y,
    );
    gizmos.sphere(
        Isometry3d::from_translation(mouse_pos),
        3.0,
        Color::srgb(1.0, 1.0, 0.0),
    );
}
