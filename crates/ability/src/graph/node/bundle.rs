use super::{EffectNodeExecuteState, EffectNodeId, StateEffectNode};

use bevy::prelude::*;
use uuid::Uuid;

/// state effect ndoe bundle
#[derive(Debug, Default, Bundle)]
pub struct StateEffectNodeBundle<T: Component + StateEffectNode> {
    pub state_node: T,
    pub execute_state: EffectNodeExecuteState,
    pub node_id: EffectNodeId,
}

#[derive(Debug, Default, Reflect)]
pub struct InstantEffectNodeBase {
    pub node_id: Uuid,
}

impl InstantEffectNodeBase {
    pub fn new() -> Self {
        Self {
            node_id: Uuid::new_v4(),
        }
    }
}
