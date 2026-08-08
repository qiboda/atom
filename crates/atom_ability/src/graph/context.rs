//! Effect Graph 上下文：图实例的运行时数据（连接、引脚值、节点登记）。

use std::ops::Not;
use std::{fmt::Debug, sync::Arc};

use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashMap;
use bevy::platform::collections::hash_map::EntryRef;
use bevy::prelude::*;
use uuid::Uuid;

use super::blackboard::EffectValue;
use super::node::EffectNodeId;
use super::node::InstantEffectNode;
use super::node::pin::EffectNodePinGroup;
use super::pin::{EffectNodeExecPin, EffectNodeSlotPin, EffectNodeSlotValue};

/// 图根实体的引用：包装 [`Entity`] 以便安全传递。
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
#[reflect(Component)]
pub struct GraphRef(Entity);

impl GraphRef {
    /// 由图根实体构造引用。
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// 取回被包装的图根实体。
    pub fn get_entity(&self) -> Entity {
        self.0
    }
}

/// 即时节点注册表：以 [`Uuid`] 为键存储图中的即时节点（不占实体的节点）。
#[derive(Resource, Default)]
pub struct InstantEffectNodeMap {
    /// UUID → 即时节点。
    pub nodes: HashMap<Uuid, Arc<dyn InstantEffectNode>>,
}

impl InstantEffectNodeMap {
    /// 注册即时节点。
    pub fn insert(&mut self, uuid: Uuid, node: Arc<dyn InstantEffectNode>) {
        self.nodes.insert(uuid, node);
    }

    /// 按 UUID 查找即时节点。
    pub fn get(&self, uuid: Uuid) -> Option<Arc<dyn InstantEffectNode>> {
        self.nodes.get(&uuid).cloned()
    }
}

/// 外部上下文 trait：图之外的业务数据容器，随图实体存储。
pub trait EffectGraphExternalContext: Debug + Sync + Send {}

/// Effect Graph 运行时上下文组件：记录图内全部连接、引脚值、入口节点与节点登记。
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct EffectGraphContext {
    // output to input connections
    /// 执行流连接：输出执行口 → 后续执行口列表。
    pub exec_connections: HashMap<EffectNodeExecPin, Vec<EffectNodeExecPin>>,
    /// 数据流连接：输出数据槽 → 后续数据槽列表。
    pub slot_connections: HashMap<EffectNodeSlotPin, Vec<EffectNodeSlotPin>>,

    // output and input pin stored values
    /// 输出引脚存储值。
    pub outputs: HashMap<EffectNodeSlotPin, EffectNodeSlotValue>,
    /// 输入引脚存储值。
    pub inputs: HashMap<EffectNodeSlotPin, EffectNodeSlotValue>,

    /// 图的入口节点（执行起点）。
    pub entry_node: Option<Entity>,
    /// 图根实体引用。
    pub graph_ref: Option<GraphRef>,

    /// 图中即时节点的 UUID 列表。
    pub instant_nodes: Vec<Uuid>,
    /// 图中状态节点（占用实体的节点）列表。
    pub state_nodes: Vec<Entity>,

    // 方式1，是将external context存入Res中，使用id去获取，但删除比较麻烦
    // 方式2，将external context定义为一个trait，然后在context中存储一个Option<Box<dyn Trait>>，这样可以直接存储
    // 方式3，将external context定义为一个trait，且实现为一个component, 插入这个component到Effeect Graph, 通过trait查询这个组件。
    // 方式4，感觉没什么用，删除。
    /// 图的外部业务上下文（可选，不参与反射）。
    #[reflect(ignore)]
    pub external_context: Option<Box<dyn EffectGraphExternalContext>>,
}

impl EffectGraphContext {
    /// 创建空上下文。
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

    /// 用新实体替换旧状态节点实体：重写所有以旧实体为节点的连接与入口。
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
                if let Some(value) = self.outputs.remove(&key) {
                    key.node_id = new_entity.into();
                    self.outputs.insert(key, value);
                }
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
                if let Some(value) = self.inputs.remove(&key) {
                    key.node_id = EffectNodeId::Entity(new_entity);
                    self.inputs.insert(key, value);
                }
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
                if let Some(value) = self.exec_connections.remove(&key) {
                    key.node_id = EffectNodeId::Entity(new_entity);
                    self.exec_connections.insert(key, value);
                }
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
                if let Some(value) = self.slot_connections.remove(&key) {
                    key.node_id = EffectNodeId::Entity(new_entity);
                    self.slot_connections.insert(key, value);
                }
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
    /// 取指定输入槽的可变引用。
    pub fn get_input_value_mut(
        &mut self,
        key: &EffectNodeSlotPin,
    ) -> Option<&mut EffectNodeSlotValue> {
        self.inputs.get_mut(key)
    }

    /// 取指定输入槽的不可变引用。
    pub fn get_input_value(&self, key: &EffectNodeSlotPin) -> Option<&EffectNodeSlotValue> {
        self.inputs.get(key)
    }

    /// 按引脚名取输入值并转换为类型 `T`；`Ref` 引用会被递归解析为实际值。
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

    /// 取指定输入槽的值并转换为类型 `T`；`Ref` 引用会被递归解析为实际值。
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

    /// 取指定输出槽的可变引用。
    pub fn get_output_value_mut(
        &mut self,
        key: &EffectNodeSlotPin,
    ) -> Option<&mut EffectNodeSlotValue> {
        self.outputs.get_mut(key)
    }

    /// 取指定输出槽的不可变引用。
    pub fn get_output_value(&self, key: &EffectNodeSlotPin) -> Option<&EffectNodeSlotValue> {
        self.outputs.get(key)
    }
}

impl EffectGraphContext {
    /// 添加执行流连接：向 `key` 输出口追加后续执行口。
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

    /// 添加数据流连接：向 `key` 输出槽追加后续数据槽。
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

    /// 取输出执行口连接到的后续执行口列表。
    pub fn get_connected_output_exec_pins(
        &self,
        key: &EffectNodeExecPin,
    ) -> Option<&Vec<EffectNodeExecPin>> {
        self.exec_connections.get(key)
    }
}

impl EffectGraphContext {
    /// 写入输入槽的值。
    pub fn insert_input_value(&mut self, key: EffectNodeSlotPin, value: EffectNodeSlotValue) {
        self.inputs.insert(key, value);
    }

    /// 写入输出槽的值。
    pub fn insert_output_value(&mut self, key: EffectNodeSlotPin, value: EffectNodeSlotValue) {
        self.outputs.insert(key, value);
    }
}

impl EffectGraphContext {
    /// 登记状态节点（不重复插入）。
    pub fn insert_state_node(&mut self, node: Entity) {
        assert!(self.state_nodes.iter().any(|entity| entity == &node).not());
        self.state_nodes.push(node);
    }

    /// 登记即时节点 UUID（不重复插入）。
    pub fn insert_instant_node(&mut self, node: Uuid) {
        assert!(self.instant_nodes.iter().any(|uuid| uuid == &node).not());
        self.instant_nodes.push(node);
    }
}

impl EffectGraphContext {
    /// 取入口节点。
    pub fn get_entry_node(&self) -> Option<Entity> {
        self.entry_node
    }

    /// 设置入口节点。
    pub fn set_entry_node(&mut self, node: Entity) {
        self.entry_node = Some(node);
    }

    /// 设置图根实体引用。
    pub fn set_graph_ref(&mut self, graph_ref: GraphRef) {
        self.graph_ref = Some(graph_ref);
    }

    /// 取图根实体引用。
    pub fn get_graph_ref(&self) -> Option<GraphRef> {
        self.graph_ref
    }
}
