use std::path::PathBuf;

use crate::{building::BuildingStats, combat::component::UnitBattleStats, unit::component::UnitStats};
use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Deserialize, Serialize)]
pub enum ThingType {
    AttackerGun,
    AttackerCannon,
    Wall,
    ProjWinged,
    SupplyStorage,
}

impl ThingType {
    pub fn get_path(&self) -> PathBuf {
        PathBuf::from("assets").join(self.get_assets_path())
    }

    pub fn get_assets_path(&self) -> PathBuf {
        match self {
            ThingType::AttackerGun => PathBuf::from("attackerGun"),
            ThingType::AttackerCannon => PathBuf::from("attackerCannon"),
            ThingType::Wall => PathBuf::from("wall"),
            ThingType::ProjWinged => PathBuf::from("projectiles/projWinged"),
            ThingType::SupplyStorage => PathBuf::from("buildings/supplyStorage"),
        }
    }

    pub fn get_info(&self) -> ThingInfo {
        let info_path = self.get_path().join("info.toml");
        let text = std::fs::read_to_string(info_path).expect("Failed to read thing info file");
        let info: ThingInfo = toml::from_str(&text).expect("Failed to parse thing info file");
        if let Some(stats) = info.building_stats {
            stats.validate().expect("Invalid building stats");
        }
        info
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, ThingType::AttackerGun | ThingType::AttackerCannon)
    }
}

#[derive(Resource, Debug)]
pub struct ThingCatalog {
    things: HashMap<ThingType, ThingInfo>,
}

impl ThingCatalog {
    pub fn new() -> Self {
        let mut things = HashMap::new();
        for thing_type in ThingType::iter() {
            let info = thing_type.get_info();
            if info.t_type != thing_type {
                panic!(
                    "ThingType mismatch: expected {:?}, found {:?}",
                    thing_type, info.t_type
                );
            }
            things.insert(thing_type, info);
        }
        Self { things }
    }

    pub fn get_info(&self, thing_type: ThingType) -> Option<&ThingInfo> {
        self.things.get(&thing_type)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThingInfo {
    pub t_type: ThingType,
    pub unit_stats: Option<UnitStats>,
    pub battle_stats: Option<UnitBattleStats>,
    pub building_stats: Option<BuildingStats>,
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
        assert_eq!(info.building_stats.unwrap().size, UVec2::splat(3));

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
