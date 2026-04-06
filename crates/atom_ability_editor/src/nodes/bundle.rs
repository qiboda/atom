use bevy::prelude::Bundle;

use super::{node::EffectNodeState, pin::EffectNodeInput, pin::EffectNodeOutput};

#[derive(Debug, Bundle, Default)]
pub struct EffectNodeBaseBundle {
    pub effect_node_state: EffectNodeState,
    pub effect_node_inputs: EffectNodeInput,
    pub effect_node_outputs: EffectNodeOutput,
}
