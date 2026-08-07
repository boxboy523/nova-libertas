use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct AnimationSet {
    pub stand: AnimationData,
    pub moving: Option<AnimationData>,
    pub attacking: Option<AnimationData>,
}

impl AnimationSet {
    pub fn get_data(&self, kind: AnimationKind) -> &AnimationData {
        match kind {
            AnimationKind::Stand => &self.stand,
            AnimationKind::Move => self.moving.as_ref().unwrap_or(&self.stand),
            AnimationKind::Attack => self.attacking.as_ref().unwrap_or(&self.stand),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub material: Handle<StandardMaterial>,
    pub frame_meshes: Vec<AnimationFrameMesh>,
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub fps: f32,
    pub looping: bool,
}

#[derive(Debug, Clone)]
pub struct AnimationFrameMesh {
    pub normal: Handle<Mesh>,
    pub flipped: Handle<Mesh>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Stand,
    Move,
    Attack,
}

#[derive(Component, Debug, Clone)]
pub struct CurrentAnimation(pub AnimationKind);

#[derive(Component, Debug, Clone, Default)]
pub struct AnimationState {
    pub timer: Timer,
    pub frame: u32,
    pub frame_count: u32,
    pub dir_idx: u32,
    pub columns: u32,
    pub rows: u32,
    pub looping: bool,
    pub fliped: bool,
}

impl AnimationState {
    pub fn from_data(data: &AnimationData) -> Self {
        AnimationState {
            timer: Timer::from_seconds(1.0 / data.fps, TimerMode::Repeating),
            frame_count: data.frame_count,
            columns: data.columns,
            rows: data.rows,
            looping: data.looping,
            ..default()
        }
    }
}
