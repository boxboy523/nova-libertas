use bevy::prelude::*;

use crate::ui::components::HpBarRef;

pub fn remove_hp_bar(
    trigger: On<Remove, HpBarRef>,
    query: Query<&HpBarRef>,
    mut commands: Commands,
) {
    if let Ok(hp_bar) = query.get(trigger.entity) {
        commands.entity(hp_bar.root).despawn();
    } else {
        warn!(
            "Failed to find HpBarRef for entity {:?} when trying to remove hp bar",
            trigger.entity
        );
    }
}
