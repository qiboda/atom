use std::ops::Not;

use bevy::{prelude::*, utils::HashMap};

use super::{blackboard::EffectValue, node::EffectNodeUuid};

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
pub struct GraphRef(Entity);

impl GraphRef {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn get_entity(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Hash)]
pub struct EffectPinKey {
    pub node: Entity,
    pub node_id: EffectNodeUuid,
    pub key: &'static str,
}

#[derive(Debug, Component, Default)]
pub struct EffectGraphContext {
    pub blackboard: HashMap<Name, EffectValue>,

    pub outputs: HashMap<EffectPinKey, EffectValue>,

    pub inputs: HashMap<EffectPinKey, EffectValue>,

    pub entry_node: Option<Entity>,

    // bool is active or not
    pub nodes: Vec<Entity>,
}

impl EffectGraphContext {
    pub fn new() -> Self {
        Self {
            blackboard: HashMap::default(),
            outputs: HashMap::default(),
            inputs: HashMap::default(),
            entry_node: None,
            nodes: vec![],
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

    pub fn get_output_value_mut(&mut self, key: &EffectPinKey) -> Option<&mut EffectValue> {
        self.outputs.get_mut(key)
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

impl EffectGraphContext {
    pub fn insert_node(&mut self, node: Entity) {
        assert!(self.nodes.iter().any(|entity| entity == &node).not());
        self.nodes.push(node);
    }
}
