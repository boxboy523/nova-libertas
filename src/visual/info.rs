use crate::prelude::*;
use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpriteConfig {
    pub sprite_info: SpriteInfo,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpriteInfo {
    #[serde(flatten)]
    pub kind: SpriteInfoKind,
    pub size: Vec2,
    #[serde(default)]
    pub offset: Vec2, // 스프라이트 오프셋
    pub anchor: Option<VisualAnchor>, // 시각적 앵커 포인트
    #[serde(default)]
    pub roll_offset_degrees: f32, // 회전 오프셋 각도 (도 단위)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpriteInfoKind {
    Simple {
        file: PathBuf, // 단순 이미지 파일 경로
    },
    AnimationSet {
        animations: HashMap<AnimationKind, AnimationClipInfo>, // 애니메이션 클립 정보
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnimationClipInfo {
    pub file: Option<PathBuf>, // 애니메이션 이미지 파일 경로
    pub cell_size: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub fps: f32,
    pub looping: bool, // 애니메이션이 반복되는지 여부
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct VisualAnchor {
    pub muzzle: Vec2,
    pub hit: Vec2,
}
