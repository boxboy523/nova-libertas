use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpriteInfo {
    pub simple: Option<SimpleSpriteInfo>,
    pub animation_set: Option<AnimationSetInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimpleSpriteInfo {
    pub file: PathBuf, // 스프라이트 이미지 파일 경로
    pub size: Vec2,    // 스프라이트의 크기
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnimationSetInfo {
    pub size: Vec2,
    pub stand: AnimationClipInfo,
    pub moving: Option<AnimationClipInfo>,
    pub attacking: Option<AnimationClipInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnimationClipInfo {
    pub file: Option<PathBuf>, // 애니메이션 이미지 파일 경로
    pub cell_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub frame_count: usize,
    pub fps: f32,
    pub looping: bool, // 애니메이션이 반복되는지 여부
}

#[derive(Debug, Clone)]
pub enum Sprite {
    Simple(SimpleSprite),
    AnimationSet(AnimationSet),
}

#[derive(Debug, Clone)]
pub struct SimpleSprite {
    pub image: Handle<Image>,
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct AnimationSet {
    pub stand: AnimationClip,
    pub moving: Option<AnimationClip>,
    pub attacking: Option<AnimationClip>,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlas>,
    pub frame_count: usize,
    pub fps: f32,
    pub looping: bool,
}
