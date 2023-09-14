use bevy::prelude::Bundle;

use super::node::{EffectNodeState, EffectNodeUuid};

#[derive(Debug, Bundle, Default)]
pub struct EffectNodeBaseBundle {
    pub effect_node_state: EffectNodeState,
    pub uuid: EffectNodeUuid,
}
