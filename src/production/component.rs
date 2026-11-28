use std::collections::VecDeque;
use bevy::prelude::*;
use crate::thing::ThingId;

#[derive(Component, Debug, Default)]
pub struct ProductionQueue {
    pub pending: VecDeque<ThingId>,
    pub active: Vec<ProductionTask>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProductionTask {
    pub id: ThingId,
    pub remaining: f32,
}
