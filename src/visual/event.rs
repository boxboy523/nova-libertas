use bevy::prelude::*;

use crate::{thing::ThingType, unit::component::Team, visual::{SpriteCatalog, UnitVisualKind, data::{AnimationKind, AnimationState}, team_color::TeamColorMaterial}};

#[derive(Event)]
pub struct ChangeVisualEvent {
    pub visual_entity: Entity,
    pub facing_oct: Option<u32>,
    pub t_type: ThingType,
    pub team: Team,
    pub animation_kind: Option<AnimationKind>,
}

pub fn change_visual_trigger(
    event: On<ChangeVisualEvent>,
    mut commands: Commands,
    mut query: Query<(Option<&mut AnimationState>, &mut MeshMaterial3d<TeamColorMaterial>,)>,
    catalog: Res<SpriteCatalog>,
) {
    let Ok((mut anim_state_opt, mut material)) = query.get_mut(event.visual_entity) else {
        warn!("Entity {:?} does not have AnimationState component", event.visual_entity);
        return;
    };

    let mut new_state = None;
    let anim_state = if let Some(anim_state) = anim_state_opt.as_mut() {
        anim_state
    } else {
        new_state = Some(AnimationState::default());
        new_state.as_mut().unwrap()
    };

    let animation_kind = if let Some(kind) = event.animation_kind {
        anim_state.kind = kind;
        kind
    } else {
        anim_state.kind
    };

    let Some(anim_data) = catalog.get_data(event.t_type, animation_kind) else {
        warn!(
            "Animation data for ThingType {:?} and AnimationKind {:?} not found in catalog",
            event.t_type, animation_kind
        );
        return;
    };

    if let Some(facing_oct) = event.facing_oct {
        anim_state.facing_oct = facing_oct;
        anim_state.mesh_dirty = true;
    }
    if event.animation_kind.is_some() {
        anim_state.update_from_data(anim_data);
        *material = anim_data
            .material
            .get(&event.team)
            .expect("Material not found for team")
            .clone().into();
    }

    if let Some(new_state) = new_state {
        commands.entity(event.visual_entity).insert(new_state);
    }
}
