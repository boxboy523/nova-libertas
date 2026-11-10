use crate::visual::data::AnimationKind;
use std::{collections::HashMap, path::{Path, PathBuf}};

use anyhow::Context;
use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpriteConfig {
    pub sprite_info: SpriteInfo,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpriteInfo {
    #[serde(flatten)]
    pub kind: SpriteInfoKind,
    /// World units per source pixel (per animation frame, not the full sheet).
    pub scale: f32,
    #[serde(default)]
    pub offset: Vec2, // Sprite-plane offset in world units; not multiplied by scale.
    /// Render-only translation along world X/Y/Z, independent of billboard rotation.
    #[serde(default)]
    pub world_offset: Vec3,
    pub anchor: Option<VisualAnchor>, // 시각적 앵커 포인트
    #[serde(default)]
    pub roll_offset_degrees: f32, // 회전 오프셋 각도 (도 단위)
}

impl SpriteInfo {
    pub fn scaled_size(&self, pixel_size: UVec2) -> anyhow::Result<Vec2> {
        anyhow::ensure!(self.scale.is_finite() && self.scale > 0.0, "Sprite scale must be finite and positive");
        anyhow::ensure!(pixel_size.x > 0 && pixel_size.y > 0, "Sprite pixel size must be nonzero");
        let size = pixel_size.as_vec2() * self.scale;
        anyhow::ensure!(size.is_finite(), "Scaled sprite size is too large");
        Ok(size)
    }

    /// Resolve the catalog's representative world size synchronously at startup.
    /// Animation meshes use each clip's cell_size; stand determines catalog size.
    pub fn world_size(&self, asset_dir: &Path) -> anyhow::Result<Vec2> {
        let pixel_size = match &self.kind {
            SpriteInfoKind::Simple { file } => {
                let path = asset_dir.join(file);
                let extension = path.extension().and_then(|value| value.to_str())
                    .context("Sprite image needs a file extension")?;
                let bytes = std::fs::read(&path)
                    .with_context(|| format!("Failed to read sprite image {}", path.display()))?;
                // Reuse Bevy's enabled image decoders; no additional image dependency.
                Image::from_buffer(
                    &bytes,
                    ImageType::Extension(extension),
                    CompressedImageFormats::NONE,
                    true,
                    ImageSampler::default(),
                    RenderAssetUsages::default(),
                ).map_err(|error| anyhow::anyhow!("Failed to decode {}: {error}", path.display()))?
                    .size()
            }
            SpriteInfoKind::AnimationSet { animations } => {
                animations.get(&AnimationKind::Stand)
                    .context("AnimationSet must contain a stand animation")?
                    .cell_size
            }
        };
        self.scaled_size(pixel_size)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thing::ThingType;
    use strum::IntoEnumIterator;

    fn sprite_info(thing_type: ThingType) -> SpriteInfo {
        let text = std::fs::read_to_string(thing_type.get_path().join("info.toml")).unwrap();
        toml::from_str::<SpriteConfig>(&text).unwrap().sprite_info
    }

    #[test]
    fn animation_world_sizes_use_source_frames_not_sheets() {
        for (thing_type, expected) in [
            (ThingType::AttackerGun, 29.4),
            (ThingType::AttackerCannon, 43.2),
        ] {
            let info = sprite_info(thing_type);
            let size = info.world_size(&thing_type.get_path()).unwrap();
            assert!((size - Vec2::splat(expected)).length() < 0.001);
            let SpriteInfoKind::AnimationSet { animations } = &info.kind else {
                panic!("Expected animated unit");
            };
            for clip in animations.values() {
                assert!((info.scaled_size(clip.cell_size).unwrap() - size).length() < 0.001);
            }
        }
    }

    #[test]
    fn all_sprite_configs_resolve_positive_scaled_sizes() {
        for thing_type in ThingType::iter() {
            let info = sprite_info(thing_type);
            assert_eq!(info.scale, 0.15);
            let size = info.world_size(&thing_type.get_path())
                .unwrap_or_else(|error| panic!("{thing_type:?}: {error:#}"));
            assert!(size.is_finite() && size.x > 0.0 && size.y > 0.0);
        }
    }

    #[test]
    fn scale_preserves_aspect_ratio_and_rejects_invalid_values() {
        let mut info = sprite_info(ThingType::AttackerGun);
        let size = info.scaled_size(UVec2::new(100, 200)).unwrap();
        assert!((size - Vec2::new(15.0, 30.0)).length() < 0.001);
        assert!(info.scaled_size(UVec2::new(0, 100)).is_err());
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            info.scale = scale;
            assert!(info.scaled_size(UVec2::ONE).is_err());
        }
    }

    #[test]
    fn world_offset_defaults_to_zero_and_reads_world_axes() {
        let text = r#"
            [sprite_info]
            type = "simple"
            file = "sprite.png"
            scale = 0.15
        "#;
        let defaults: SpriteConfig = toml::from_str(text).unwrap();
        assert_eq!(defaults.sprite_info.world_offset, Vec3::ZERO);
        let configured: SpriteConfig = toml::from_str(&format!(
            "{text}\nworld_offset = [3.0, 10.0, -5.0]\noffset = [1.0, -2.0]\n"
        )).unwrap();
        assert_eq!(configured.sprite_info.world_offset, Vec3::new(3.0, 10.0, -5.0));
        assert_eq!(configured.sprite_info.offset, Vec2::new(1.0, -2.0));
        let size = configured.sprite_info.scaled_size(UVec2::splat(100)).unwrap();
        assert!((size - Vec2::splat(15.0)).length() < 0.001);
    }

    #[test]
    fn offsets_and_anchors_remain_world_space() {
        let info = sprite_info(ThingType::AttackerGun);
        assert_eq!(info.offset, Vec2::new(0.0, -7.0));
        let anchor = info.anchor.unwrap();
        assert_eq!(anchor.muzzle, Vec2::new(0.0, 20.0));
        assert_eq!(anchor.hit, Vec2::new(0.0, 20.0));
    }
}
