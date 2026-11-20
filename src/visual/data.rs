use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{unit::component::Team, visual::team_color::TeamColorMaterial};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationKind {
    #[default]
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
    pub kind: AnimationKind,
    pub facing_oct: u32,
    pub mesh_dirty: bool,
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

    pub fn update_from_data(&mut self, data: &AnimationData) {
        self.frame_count = data.frame_count;
        self.frame = 0;
        self.timer = Timer::from_seconds(1.0 / data.fps, TimerMode::Repeating);
        self.columns = data.columns;
        self.rows = data.rows;
        self.looping = data.looping;
        self.mesh_dirty = true;
    }

    pub fn update_fps(&mut self, fps: f32) {
        self.timer.set_duration(std::time::Duration::from_secs_f32(1.0 / fps));
    }

    pub fn tick(&mut self, delta: f32) {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        if self.timer.just_finished() {
            self.frame += 1;
            if self.frame >= self.frame_count {
                if self.looping {
                    self.frame = 0;
                } else {
                    self.frame = self.frame_count - 1;
                }
            }
            self.mesh_dirty = true;
        }
    }

    pub fn get_material_handle(
        &self,
        data: &AnimationData,
        team: Team,
    ) -> Handle<TeamColorMaterial> {
        data.material
            .get(&team)
            .expect("Material not found for team")
            .clone()
    }
}
