use std::path::PathBuf;

use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Deserialize, Serialize)]
pub enum ThingType {
    AttackerGun,
    Wall,
}

impl ThingType {
    pub fn get_path(&self) -> PathBuf {
        PathBuf::from("assets").join(self.get_assets_path())
    }

    pub fn get_assets_path(&self) -> PathBuf {
        match self {
            ThingType::AttackerGun => PathBuf::from("attackerGun"),
            ThingType::Wall => PathBuf::from("wall"),
        }
    }

    pub fn get_info(&self) -> ThingInfo {
        let info_path = self.get_path().join("info.toml");
        let text = std::fs::read_to_string(info_path).expect("Failed to read thing info file");
        toml::from_str(&text).expect("Failed to parse thing info file")
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
}
