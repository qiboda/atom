use std::ops::Not;
use std::{fmt::Debug, sync::Arc};

use bevy::ecs::entity::EntityHashMap;
use bevy::utils::hashbrown::hash_map::EntryRef;
use bevy::{prelude::*, utils::HashMap};
use uuid::Uuid;

use super::blackboard::EffectValue;
use super::node::pin::EffectNodePinGroup;
use super::node::EffectNodeId;
use super::node::InstantEffectNode;
use super::pin::{EffectNodeExecPin, EffectNodeSlotPin, EffectNodeSlotValue};

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
#[reflect(Component)]
pub struct GraphRef(Entity);

impl GraphRef {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn get_entity(&self) -> Entity {
        self.0
    }
}

#[derive(Resource, Default)]
pub struct InstantEffectNodeMap {
    pub nodes: HashMap<Uuid, Arc<dyn InstantEffectNode>>,
}

impl InstantEffectNodeMap {
    pub fn insert(&mut self, uuid: Uuid, node: Arc<dyn InstantEffectNode>) {
        self.nodes.insert(uuid, node);
    }

    pub fn get(&self, uuid: Uuid) -> Option<Arc<dyn InstantEffectNode>> {
        self.nodes.get(&uuid).cloned()
    }
}

pub trait EffectGraphExternalContext: Debug + Sync + Send {}

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct EffectGraphContext {
    // output to input connections
    pub exec_connections: HashMap<EffectNodeExecPin, Vec<EffectNodeExecPin>>,
    pub slot_connections: HashMap<EffectNodeSlotPin, Vec<EffectNodeSlotPin>>,

    // output and input pin stored values
    pub outputs: HashMap<EffectNodeSlotPin, EffectNodeSlotValue>,
    pub inputs: HashMap<EffectNodeSlotPin, EffectNodeSlotValue>,

    pub entry_node: Option<Entity>,
    pub graph_ref: Option<GraphRef>,

    pub instant_nodes: Vec<Uuid>,
    pub state_nodes: Vec<Entity>,

    // 方式1，是将external context存入Res中，使用id去获取，但删除比较麻烦
    // 方式2，将external context定义为一个trait，然后在context中存储一个Option<Box<dyn Trait>>，这样可以直接存储
    // 方式3，将external context定义为一个trait，且实现为一个component, 插入这个component到Effeect Graph, 通过trait查询这个组件。
    // 方式4，感觉没什么用，删除。
    #[reflect(ignore)]
    pub external_context: Option<Box<dyn EffectGraphExternalContext>>,
}

impl EffectGraphContext {
    pub fn new() -> Self {
        Self {
            exec_connections: HashMap::default(),
            slot_connections: HashMap::default(),
            outputs: HashMap::default(),
            inputs: HashMap::default(),
            entry_node: None,
            instant_nodes: vec![],
            state_nodes: vec![],
            external_context: None,
            graph_ref: None,
        }
    }

    pub fn replace_state_entities(&mut self, new_entities: EntityHashMap<Entity>) {
        self.state_nodes.clear();

        for (old_entity, new_entity) in new_entities {
            let keys = self
                .outputs
                .keys()
                .filter(|key| match key.node_id {
                    EffectNodeId::Uuid(_) => false,
                    EffectNodeId::Entity(entity) => entity == old_entity,
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut key in keys {
                let value = self.outputs.remove(&key).unwrap();
                key.node_id = new_entity.into();
                self.outputs.insert(key, value);
            }

            let keys = self
                .inputs
                .keys()
                .filter(|key| match key.node_id {
                    EffectNodeId::Uuid(_) => false,
                    EffectNodeId::Entity(entity) => entity == old_entity,
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut key in keys {
                let value = self.inputs.remove(&key).unwrap();
                key.node_id = EffectNodeId::Entity(new_entity);
                self.inputs.insert(key, value);
            }

            let keys = self
                .exec_connections
                .keys()
                .filter(|key| match key.node_id {
                    EffectNodeId::Uuid(_) => false,
                    EffectNodeId::Entity(entity) => entity == old_entity,
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut key in keys {
                let value = self.exec_connections.remove(&key).unwrap();
                key.node_id = EffectNodeId::Entity(new_entity);
                self.exec_connections.insert(key, value);
            }
            self.exec_connections.values_mut().for_each(|value| {
                for v in value {
                    if v.node_id == EffectNodeId::Entity(old_entity) {
                        v.node_id = EffectNodeId::Entity(new_entity);
                    }
                }
            });

            let keys = self
                .slot_connections
                .keys()
                .filter(|key| match key.node_id {
                    EffectNodeId::Uuid(_) => false,
                    EffectNodeId::Entity(entity) => entity == old_entity,
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut key in keys {
                let value = self.slot_connections.remove(&key).unwrap();
                key.node_id = EffectNodeId::Entity(new_entity);
                self.slot_connections.insert(key, value);
            }
            self.slot_connections.values_mut().for_each(|value| {
                for v in value {
                    if v.node_id == EffectNodeId::Entity(old_entity) {
                        v.node_id = EffectNodeId::Entity(new_entity);
                    }
                }
            });

            if self.entry_node == Some(old_entity) {
                self.entry_node = Some(new_entity);
            }

            self.state_nodes.push(new_entity);
        }
    }
}

impl EffectGraphContext {
    pub fn get_input_value_mut(
        &mut self,
        key: &EffectNodeSlotPin,
    ) -> Option<&mut EffectNodeSlotValue> {
        self.inputs.get_mut(key)
    }

    pub fn get_input_value(&self, key: &EffectNodeSlotPin) -> Option<&EffectNodeSlotValue> {
        self.inputs.get(key)
    }

    pub fn get_input_value_type_from_node<'a, T: TryFrom<&'a EffectValue>>(
        &'a self,
        entity: Entity,
        node: &impl EffectNodePinGroup,
        pin_name: &'static str,
    ) -> Option<T>
    where
        <T as TryFrom<&'a EffectValue>>::Error: Debug,
    {
        node.get_input_slot_pin_by_name(pin_name)
            .and_then(|slot_pin| {
                self.get_input_value_type::<T>(&EffectNodeSlotPin {
                    node_id: EffectNodeId::Entity(entity),
                    slot: *slot_pin,
                })
            })
    }

    pub fn get_input_value_type<'a, T: TryFrom<&'a EffectValue>>(
        &'a self,
        key: &EffectNodeSlotPin,
    ) -> Option<T>
    where
        <T as TryFrom<&'a EffectValue>>::Error: Debug,
    {
        let slot_value = self.get_input_value(key);

        match slot_value {
            Some(value) => match value {
                EffectNodeSlotValue::Value(value) => match value.try_into() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        error!("convert key: {:?}, {:?}", key, e);
                        None
                    }
                },

                EffectNodeSlotValue::Ref(slot_pin) => {
                    let slot_value = self.get_input_value(slot_pin);
                    match slot_value {
                        Some(EffectNodeSlotValue::Value(value)) => match value.try_into() {
                            Ok(v) => Some(v),
                            Err(e) => {
                                error!("convert key: {:?}, {:?}", key, e);
                                None
                            }
                        },
                        Some(EffectNodeSlotValue::Ref(_)) => {
                            error!("output node pin {:?} can not ref value", slot_pin);
                            None
                        }
                        None => {
                            error!("node pin value {:?} not found", slot_pin);
                            None
                        }
                    }
                }
            },
            None => {
                error!("node pin value {:?} not found", key);
                None
            }
        }
    }

    pub fn get_output_value_mut(
        &mut self,
        key: &EffectNodeSlotPin,
    ) -> Option<&mut EffectNodeSlotValue> {
        self.outputs.get_mut(key)
    }

    pub fn get_output_value(&self, key: &EffectNodeSlotPin) -> Option<&EffectNodeSlotValue> {
        self.outputs.get(key)
    }
}

impl EffectGraphContext {
    pub fn add_exec_connection(&mut self, key: EffectNodeExecPin, value: &[EffectNodeExecPin]) {
        match self.exec_connections.entry_ref(&key) {
            EntryRef::Occupied(entry) => {
                entry.into_mut().extend(value);
            }
            EntryRef::Vacant(_) => {
                self.exec_connections.insert(key, value.to_vec());
            }
        }
    }

    pub fn add_slot_connection(&mut self, key: EffectNodeSlotPin, value: &[EffectNodeSlotPin]) {
        match self.slot_connections.entry_ref(&key) {
            EntryRef::Occupied(entry) => {
                entry.into_mut().extend(value);
            }
            EntryRef::Vacant(_) => {
                self.slot_connections.insert(key, value.to_vec());
            }
        }
    }

    pub fn get_connected_output_exec_pins(
        &self,
        key: &EffectNodeExecPin,
    ) -> Option<&Vec<EffectNodeExecPin>> {
        self.exec_connections.get(key)
    }
}

impl EffectGraphContext {
    pub fn insert_input_value(&mut self, key: EffectNodeSlotPin, value: EffectNodeSlotValue) {
        self.inputs.insert(key, value);
    }

    pub fn insert_output_value(&mut self, key: EffectNodeSlotPin, value: EffectNodeSlotValue) {
        self.outputs.insert(key, value);
    }
}

impl EffectGraphContext {
    pub fn insert_state_node(&mut self, node: Entity) {
        assert!(self.state_nodes.iter().any(|entity| entity == &node).not());
        self.state_nodes.push(node);
    }

    pub fn insert_instant_node(&mut self, node: Uuid) {
        assert!(self.instant_nodes.iter().any(|uuid| uuid == &node).not());
        self.instant_nodes.push(node);
    }
}

impl EffectGraphContext {
    pub fn get_entry_node(&self) -> Option<Entity> {
        self.entry_node
    }

    pub fn set_entry_node(&mut self, node: Entity) {
        self.entry_node = Some(node);
    }

    pub fn set_graph_ref(&mut self, graph_ref: GraphRef) {
        self.graph_ref = Some(graph_ref);
    }

    pub fn get_graph_ref(&self) -> Option<GraphRef> {
        self.graph_ref
    }
}
