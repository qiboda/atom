use bevy::prelude::{Bundle, Component};

use super::{
    node::{EffectNode, EffectNodeState},
    pin::{EffectNodeInputs, EffectNodeOutputs},
};

#[derive(Debug, Bundle)]
pub struct EffectNodeBundle<T: EffectNode + Component> {
    effect_node: T,
    effect_node_state: EffectNodeState,
    effect_node_outputs: EffectNodeOutputs,
    effect_node_inputs: EffectNodeInputs,
}
