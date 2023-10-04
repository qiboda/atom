use std::ops::Not;

use bevy::{prelude::*, utils::HashMap};

use super::{blackboard::EffectValue, event::EffectNodeEvent, node::EffectNodeUuid};

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
    pub node_uuid: EffectNodeUuid,
    pub key: &'static str,
}

impl EffectPinKey {
    pub fn new(node: Entity, node_uuid: EffectNodeUuid, key: &'static str) -> Self {
        Self {
            node,
            node_uuid,
            key,
        }
    }
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

impl EffectGraphContext {
    pub fn get_entry_node(&self) -> Option<Entity> {
        self.entry_node
    }

    pub fn set_entry_node(&mut self, node: Entity) {
        self.entry_node = Some(node);
    }
}

impl EffectGraphContext {
    pub fn exec_next_nodes<T: EffectNodeEvent + Event>(
        &self,
        node: Entity,
        node_uuid: EffectNodeUuid,
        exec_key: &'static str,
        event_writer: &mut EventWriter<T>,
    ) {
        let key = EffectPinKey {
            node,
            node_uuid,
            key: exec_key,
        };
        if let Some(EffectValue::Vec(effect_vec)) = self.get_output_value(&key) {
            for effect in effect_vec {
                if let EffectValue::Entity(entity) = effect {
                    event_writer.send(T::new(*entity));
                }
            }
        }
    }
}
