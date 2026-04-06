use bevy::prelude::{Bundle, Component};

use super::{
    builder::EffectGraph,
    context::EffectGraphContext,
    state::{EffectGraphState, EffectGraphTickState},
};

#[derive(Debug, Bundle, Default)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component + Default> {
    pub context: EffectGraphContext,
    pub state: EffectGraphState,
    pub tick_state: EffectGraphTickState,
    pub graph: EffectGraphType,
}
