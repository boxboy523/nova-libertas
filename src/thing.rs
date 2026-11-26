use std::{fs, path::{Path, PathBuf}};

use crate::{building::BuildingStats, combat::component::UnitBattleStats, unit::component::UnitStats};
use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String")]
pub struct ThingId(u64);

impl From<String> for ThingId {
    fn from(name: String) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(&name))
    }
}

impl ThingId {
    pub const fn from_name(name: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(name))
    }
}

#[derive(Debug, Clone)]
pub struct ThingEntry {
    pub name: String,
    pub asset_dir: PathBuf,
    pub info: ThingInfo,
}

#[derive(Resource, Debug)]
pub struct ThingCatalog {
    pub asset_root: PathBuf,
    things: HashMap<ThingId, ThingEntry>,
}

impl ThingCatalog {
    pub fn new(asset_root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut catalog = Self {
            things: HashMap::new(),
            asset_root: asset_root.as_ref().to_path_buf(),
        };
        catalog.load()?;
        Ok(catalog)
    }
    pub fn load(&mut self) -> anyhow::Result<()> {
        let mut things: HashMap<ThingId, ThingEntry> = HashMap::new();

        [("units", "unit"), ("buildings", "building"), ("projectiles", "projectile")]
            .iter()
            .try_for_each(|(dir, category)| -> anyhow::Result<()> {
                let root = self.asset_root.join(dir);
                let dirs = fs::read_dir(&root)?;

                dirs.into_iter().try_for_each(|entry| -> anyhow::Result<()> {
                    let entry = entry?;
                    if !entry.file_type()?.is_dir() {
                        return Ok(());
                    }
                    let info_path = entry.path().join("info.toml");
                    let text = match fs::read_to_string(&info_path) {
                        Ok(text) => text,
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            warn!(
                                "No info.toml found for {} in {}",
                                entry.file_name().to_string_lossy(),
                                category
                            );
                            return Ok(());
                        }
                        Err(err) => return anyhow::Result::Err(err.into()),
                    };

                    let file_name = entry.file_name();
                    let dirname = file_name.to_str().ok_or(anyhow::format_err!(
                        "Failed to convert directory name to string: {:?}",
                        entry.file_name()
                    ))?;
                    let name = format!("{}:{}", category, dirname);
                    let id = ThingId::from_name(&name);
                    let info: ThingInfo = toml::from_str(&text)
                        .map_err(|e| anyhow::format_err!("Failed to parse {}: {}", name, e))?;
                    if let Some(previous) = things.get(&id) {
                        return Err(anyhow::format_err!(
                            "Duplicate ThingId {} for {} and {}",
                            id.0,
                            previous.name,
                            name
                        ));
                    }
                    things.insert(id, ThingEntry {
                        name,
                        asset_dir: Path::new(*dir).join(&file_name),
                        info,
                    });
                    Ok(())
                })?;
                Ok(())
            })?;
        self.things = things;
        Ok(())
    }

    pub fn get_info(&self, thing_id: ThingId) -> Option<&ThingInfo> {
        self.things.get(&thing_id).and_then(|entry| Some(&entry.info))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ThingId, &ThingEntry)> {
        self.things.iter()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThingInfo {
    pub unit_stats: Option<UnitStats>,
    pub battle_stats: Option<UnitBattleStats>,
    pub building_stats: Option<BuildingStats>,
    pub icon: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::info::{SpriteConfig, SpriteInfoKind};

    #[test]
    fn supply_storage_has_gameplay_and_sprite_configuration() {
        let thing_type = ThingType::SupplyStorage;
        let info = thing_type.get_info();
        assert_eq!(info.t_type, thing_type);
        assert!(!thing_type.is_unit());
        assert!(info.unit_stats.is_none());
        assert!(info.battle_stats.is_none());
        assert_eq!(info.building_stats.unwrap().size, UVec2::splat(2));

        let text = std::fs::read_to_string(thing_type.get_path().join("info.toml")).unwrap();
        let config: SpriteConfig = toml::from_str(&text).unwrap();
        let SpriteInfoKind::Simple { file } = &config.sprite_info.kind else {
            panic!("SupplyStorage must use a simple sprite");
        };
        assert_eq!(
            thing_type.get_path().join(file).canonicalize().unwrap(),
            PathBuf::from("assets/buildings/supplyStorage/supplyStorage.png").canonicalize().unwrap(),
        );
        assert_eq!(config.sprite_info.scale, 0.15);
        let size = config.sprite_info.world_size(&thing_type.get_path()).unwrap();
        assert!((size - Vec2::splat(76.8)).length() < 0.001);
    }
}
