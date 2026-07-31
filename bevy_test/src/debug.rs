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
    query_units: Query<(Entity, &Transform, &UnitStats)>,
    query_selected: Query<&Selected>,
) {
    query_units.iter().for_each(|(entity, transform, stats)| {
        let pos = transform.translation;
        let radius = stats.size;
        gizmos.circle_2d(pos.truncate(), radius, Color::srgb(0.0, 1.0, 0.0));
        if query_selected.get(entity).is_ok() {
            gizmos.circle_2d(pos.truncate(), radius + 10.0, Color::srgb(1.0, 1.0, 0.0));
        }
    });
}
