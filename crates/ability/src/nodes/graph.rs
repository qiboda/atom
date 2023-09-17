use bevy::{prelude::*, utils::HashMap};

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
    pub key: &'static str,
}

#[derive(Debug, Component)]
pub struct EffectGraphContext {
    pub blackboard: HashMap<Name, EffectValue>,

    pub outputs: HashMap<EffectPinKey, EffectValue>,

    pub inputs: HashMap<EffectPinKey, EffectValue>,
}

impl EffectGraphContext {
    pub fn new() -> Self {
        Self {
            blackboard: HashMap::default(),
            outputs: HashMap::default(),
            inputs: HashMap::default(),
        }
    }
}

impl EffectGraphContext {
    pub fn get_input_value_mut(&mut self, key: &EffectPinKey) -> Option<&mut EffectValue> {
        self.inputs.get_mut(key)
    }

    pub fn get_input_value(&self, key: &EffectPinKey) -> Option<&EffectValue> {
        self.inputs.get(key)
    }

    pub fn get_output_value(&self, key: &EffectPinKey) -> Option<&EffectValue> {
        self.outputs.get(key)
    }
}

impl EffectGraphContext {
    pub fn insert_input_value(&mut self, key: EffectPinKey, value: EffectValue) {
        self.inputs.insert(key, value);
    }

    pub fn insert_output_value(&mut self, key: EffectPinKey, value: EffectValue) {
        self.outputs.insert(key, value);
    }
}

#[derive(Debug, Bundle)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component> {
    pub context: EffectGraphContext,
    pub graph: EffectGraphType,
}
