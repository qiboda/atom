use bevy::prelude::{Bundle, Component};

use super::{node::{EffectNodeState, EffectNodeUuid}, graph::{EffectGraphContext, EffectGraph}};

#[derive(Debug, Bundle, Default)]
pub struct EffectNodeBaseBundle {
    pub effect_node_state: EffectNodeState,
    pub uuid: EffectNodeUuid,
}

#[derive(Debug, Bundle, Default)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component + Default> {
    pub context: EffectGraphContext,
    pub graph: EffectGraphType,
}