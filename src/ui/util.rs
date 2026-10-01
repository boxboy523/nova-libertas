use bevy::prelude::*;

use crate::thing::ThingType;

pub fn unit_to_icon(unit_type: ThingType, asset_server: &Res<AssetServer>) -> anyhow::Result<Handle<Image>> {
    if !unit_type.is_unit() {
        return Err(anyhow::anyhow!("Invalid unit type: {:?}", unit_type));
    }
    Ok(match unit_type {
        ThingType::AttackerGun => asset_server.load("ui/icons/units/iconAttackerGun.png"),
        ThingType::AttackerCannon => asset_server.load("ui/icons/units/iconAttackerCannon.png"),
        _ => unreachable!(),
    })
}
