use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct AnimationSet {
    pub animations: HashMap<AnimationKind, AnimationData>,
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub material: HashMap<Team, Handle<TeamColorMaterial>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationKind {
    Stand,
    Move,
    Attack,
}

impl AnimationKind {
    pub fn default_file(&self) -> PathBuf {
        match self {
            AnimationKind::Stand => PathBuf::from("stand.png"),
            AnimationKind::Move => PathBuf::from("move.png"),
            AnimationKind::Attack => PathBuf::from("attack.png"),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct CurrentAnimation(pub AnimationKind);

#[derive(Component, Debug, Clone, Default)]
pub struct AnimationState {
    pub timer: Timer,
    pub frame: u32,
    pub frame_count: u32,
    pub columns: u32,
    pub rows: u32,
    pub looping: bool,
    pub facing_oct: u32,
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
