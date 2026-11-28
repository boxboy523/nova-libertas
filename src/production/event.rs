use bevy::prelude::*;

use crate::{production::component::ProductionQueue, thing::{ThingCatalog, ThingId}, unit::component::Team};

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestUnitProduction {
    pub id: ThingId,
    pub team: Team,
}

pub fn request_unit_production(
    event: On<RequestUnitProduction>,
    building: Query<(&mut ProductionQueue, &Team)>,
    catalog: Res<ThingCatalog>
) {
    if event.team == Team::Player {

    }
}
