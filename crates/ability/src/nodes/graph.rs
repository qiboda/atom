use bevy::{
    prelude::*,
    utils::{HashMap, Uuid},
};

use super::{blackboard::EffectValue, node::EffectNodeUuid};

/// all children node  is graph nodes.
pub trait EffectGraph {}

pub trait EffectGraphBuilder {
    fn build(&self, commands: &mut Commands, effect_graph_context: &mut EffectGraphContext);
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Hash)]
pub struct EffectPinKey {
    pub node: Entity,
    pub node_id: EffectNodeUuid,
    pub output_key: &'static str,
}

#[derive(Debug, Component)]
pub struct EffectGraphContext {
    pub blackboard: HashMap<Name, EffectValue>,

    pub outputs: HashMap<EffectPinKey, EffectValue>,

    pub inputs: HashMap<EffectPinKey, EffectValue>,
}

#[derive(Debug, Bundle)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component> {
    pub context: EffectGraphContext,
    pub graph: EffectGraphType,
}
