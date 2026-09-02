use bevy::prelude::*;

#[derive(Component)]
pub struct HpBarRoot {
    pub owner: Entity,
    pub visual_height: f32,
}

#[derive(Component)]
pub struct HpBarFill;

#[derive(Component)]
pub struct HpBarRef {
    pub root: Entity,
    pub fill: Entity,
}
#[derive(Component, Default)]
pub struct DragSelection {
    pub start: Vec2,
    pub current: Vec2,
    pub active: bool,
}
