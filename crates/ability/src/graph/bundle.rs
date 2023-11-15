use bevy::prelude::{Bundle, Component};

use super::{
    builder::EffectGraph,
    context::EffectGraphContext,
    node::{EffectNodeExecuteState, EffectNodeTickState, EffectNodeUuid},
    state::EffectGraphState,
};

#[derive(Debug, Bundle, Default)]
pub struct EffectNodeBaseBundle {
    pub execute_state: EffectNodeExecuteState,
    pub tick_state: EffectNodeTickState,
    pub uuid: EffectNodeUuid,
}

#[derive(Debug, Bundle, Default)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component + Default> {
    pub context: EffectGraphContext,
    pub state: EffectGraphState,
    pub graph: EffectGraphType,
}
