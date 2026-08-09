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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::implement::seq::EffectNodeSeq;
    use crate::graph::node::pin::{EffectNodeExec, EffectNodeSlot};

    fn slot_pin(node: EffectNodeId, name: &'static str) -> EffectNodeSlotPin {
        EffectNodeSlotPin {
            node_id: node,
            slot: EffectNodeSlot::new::<f32>(name),
        }
    }

    fn exec_pin(node: EffectNodeId, name: &'static str) -> EffectNodeExecPin {
        EffectNodeExecPin {
            node_id: node,
            exec: EffectNodeExec { name },
        }
    }

    #[test]
    fn new_context_is_empty() {
        let context = EffectGraphContext::new();
        assert!(context.exec_connections.is_empty());
        assert!(context.slot_connections.is_empty());
        assert!(context.outputs.is_empty());
        assert!(context.inputs.is_empty());
        assert_eq!(context.entry_node, None);
        assert_eq!(context.graph_ref, None);
        assert!(context.instant_nodes.is_empty());
        assert!(context.state_nodes.is_empty());
        assert!(context.external_context.is_none());
    }

    #[test]
    fn default_context_is_empty() {
        let context = EffectGraphContext::default();
        assert!(context.exec_connections.is_empty());
        assert!(context.slot_connections.is_empty());
        assert!(context.outputs.is_empty());
        assert!(context.inputs.is_empty());
        assert_eq!(context.entry_node, None);
        assert_eq!(context.graph_ref, None);
    }

    #[test]
    fn insert_input_value_then_get_roundtrip() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(EffectNodeId::from_entity(None), "damage");
        context.insert_input_value(key, EffectValue::F32(10.0).into());

        assert_eq!(
            context.get_input_value(&key),
            Some(&EffectNodeSlotValue::Value(EffectValue::F32(10.0)))
        );
    }

    #[test]
    fn insert_input_value_overwrites_previous() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(EffectNodeId::from_entity(None), "damage");
        context.insert_input_value(key, EffectValue::F32(1.0).into());
        context.insert_input_value(key, EffectValue::F32(2.0).into());

        assert_eq!(
            context.get_input_value(&key),
            Some(&EffectNodeSlotValue::Value(EffectValue::F32(2.0)))
        );
    }

    #[test]
    fn get_input_value_missing_returns_none() {
        let context = EffectGraphContext::new();
        assert_eq!(
            context.get_input_value(&slot_pin(Entity::PLACEHOLDER.into(), "x")),
            None
        );
    }

    #[test]
    fn get_input_value_mut_allows_in_place_update() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(EffectNodeId::from_entity(None), "hp");
        context.insert_input_value(key, EffectValue::I32(10).into());

        let value = context
            .get_input_value_mut(&key)
            .expect("插入后应能取到可变引用");
        *value = EffectNodeSlotValue::Value(EffectValue::I32(20));

        assert_eq!(
            context.get_input_value(&key),
            Some(&EffectNodeSlotValue::Value(EffectValue::I32(20)))
        );
    }

    #[test]
    fn get_input_value_mut_missing_returns_none() {
        let mut context = EffectGraphContext::new();
        assert_eq!(
            context.get_input_value_mut(&slot_pin(Entity::PLACEHOLDER.into(), "x")),
            None
        );
    }

    #[test]
    fn insert_output_value_then_get_roundtrip() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(EffectNodeId::Uuid(Uuid::nil()), "result");
        context.insert_output_value(key, EffectValue::String("ok".into()).into());

        assert_eq!(
            context.get_output_value(&key),
            Some(&EffectNodeSlotValue::Value(EffectValue::String(
                "ok".into()
            )))
        );
    }

    #[test]
    fn insert_output_value_overwrites_previous() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(EffectNodeId::Uuid(Uuid::nil()), "result");
        context.insert_output_value(key, EffectValue::I32(1).into());
        context.insert_output_value(key, EffectValue::I32(2).into());

        assert_eq!(
            context.get_output_value(&key),
            Some(&EffectNodeSlotValue::Value(EffectValue::I32(2)))
        );
    }

    #[test]
    fn get_output_value_missing_returns_none() {
        let context = EffectGraphContext::new();
        assert_eq!(
            context.get_output_value(&slot_pin(Entity::PLACEHOLDER.into(), "x")),
            None
        );
    }

    #[test]
    fn get_output_value_mut_allows_in_place_update() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(EffectNodeId::Uuid(Uuid::nil()), "result");
        context.insert_output_value(key, EffectValue::F32(1.0).into());

        let value = context
            .get_output_value_mut(&key)
            .expect("插入后应能取到可变引用");
        *value = EffectNodeSlotValue::Value(EffectValue::F32(3.5));

        assert_eq!(
            context.get_output_value(&key),
            Some(&EffectNodeSlotValue::Value(EffectValue::F32(3.5)))
        );
    }

    #[test]
    fn get_output_value_mut_missing_returns_none() {
        let mut context = EffectGraphContext::new();
        assert_eq!(
            context.get_output_value_mut(&slot_pin(Entity::PLACEHOLDER.into(), "x")),
            None
        );
    }

    #[test]
    fn get_input_value_type_converts_value() {
        let mut context = EffectGraphContext::new();
        let key = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("message"),
        };
        context.insert_input_value(key, EffectValue::String("hello".into()).into());

        let value = context.get_input_value_type::<String>(&key);
        assert_eq!(value, Some("hello".to_string()));
    }

    #[test]
    fn get_input_value_type_resolves_ref() {
        let mut context = EffectGraphContext::new();
        let target = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("source"),
        };
        let alias = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("alias"),
        };
        context.insert_input_value(target, EffectValue::String("ref-value".into()).into());
        context.insert_input_value(alias, EffectNodeSlotValue::Ref(target));

        let value = context.get_input_value_type::<String>(&alias);
        assert_eq!(value, Some("ref-value".to_string()));
    }

    #[test]
    fn get_input_value_type_ref_to_ref_returns_none() {
        let mut context = EffectGraphContext::new();
        let inner = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("inner"),
        };
        let outer = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("outer"),
        };
        context.insert_input_value(inner, EffectNodeSlotValue::Ref(outer));
        context.insert_input_value(outer, EffectNodeSlotValue::Ref(inner));

        let value = context.get_input_value_type::<String>(&inner);
        assert_eq!(value, None, "Ref 指向 Ref 时必须解析失败");
    }

    #[test]
    fn get_input_value_type_missing_returns_none() {
        let context = EffectGraphContext::new();
        let key = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("message"),
        };
        let value = context.get_input_value_type::<String>(&key);
        assert_eq!(value, None);
    }

    #[test]
    fn get_input_value_type_wrong_type_returns_none() {
        let mut context = EffectGraphContext::new();
        let key = EffectNodeSlotPin {
            node_id: EffectNodeId::Uuid(Uuid::nil()),
            slot: EffectNodeSlot::new::<String>("message"),
        };
        context.insert_input_value(key, EffectValue::I32(42).into());

        let value = context.get_input_value_type::<String>(&key);
        assert_eq!(value, None, "I32 值不能转换为 String");
    }

    #[test]
    fn add_exec_connection_inserts_new_key() {
        let mut context = EffectGraphContext::new();
        let key = exec_pin(Entity::from_bits(1).into(), "finish");
        let next = exec_pin(Entity::from_bits(2).into(), "start");

        context.add_exec_connection(key, &[next]);

        assert_eq!(
            context.get_connected_output_exec_pins(&key),
            Some(&vec![next])
        );
    }

    #[test]
    fn add_exec_connection_appends_to_existing() {
        let mut context = EffectGraphContext::new();
        let key = exec_pin(Entity::from_bits(1).into(), "finish");
        let next_a = exec_pin(Entity::from_bits(2).into(), "start");
        let next_b = exec_pin(Entity::from_bits(3).into(), "start");

        context.add_exec_connection(key, &[next_a]);
        context.add_exec_connection(key, &[next_b]);

        assert_eq!(
            context.get_connected_output_exec_pins(&key),
            Some(&vec![next_a, next_b])
        );
    }

    #[test]
    fn get_connected_output_exec_pins_missing_returns_none() {
        let context = EffectGraphContext::new();
        let key = exec_pin(Entity::from_bits(1).into(), "finish");
        assert_eq!(context.get_connected_output_exec_pins(&key), None);
    }

    #[test]
    fn add_slot_connection_inserts_and_appends() {
        let mut context = EffectGraphContext::new();
        let key = slot_pin(Entity::from_bits(1).into(), "out");
        let next_a = slot_pin(Entity::from_bits(2).into(), "in");
        let next_b = slot_pin(Entity::from_bits(3).into(), "in");

        context.add_slot_connection(key, &[next_a]);
        context.add_slot_connection(key, &[next_b]);

        assert_eq!(
            context.slot_connections.get(&key),
            Some(&vec![next_a, next_b])
        );
    }

    #[test]
    fn set_and_get_entry_node() {
        let mut context = EffectGraphContext::new();
        assert_eq!(context.get_entry_node(), None);

        let node = Entity::from_bits(7);
        context.set_entry_node(node);
        assert_eq!(context.get_entry_node(), Some(node));

        let node2 = Entity::from_bits(8);
        context.set_entry_node(node2);
        assert_eq!(context.get_entry_node(), Some(node2));
    }

    #[test]
    fn set_and_get_graph_ref() {
        let mut context = EffectGraphContext::new();
        assert_eq!(context.get_graph_ref(), None);

        let graph_ref = GraphRef::new(Entity::from_bits(9));
        context.set_graph_ref(graph_ref);

        assert_eq!(context.get_graph_ref(), Some(graph_ref));
        assert_eq!(graph_ref.get_entity(), Entity::from_bits(9));
    }

    #[test]
    fn insert_state_node_dedups_registration() {
        let mut context = EffectGraphContext::new();
        let node = Entity::from_bits(1);

        context.insert_state_node(node);
        context.insert_state_node(Entity::from_bits(2));
        assert_eq!(context.state_nodes, vec![node, Entity::from_bits(2)]);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn insert_state_node_twice_panics() {
        let mut context = EffectGraphContext::new();
        let node = Entity::from_bits(1);
        context.insert_state_node(node);
        context.insert_state_node(node);
    }

    #[test]
    fn insert_instant_node_registers_uuid() {
        let mut context = EffectGraphContext::new();
        let uuid_a = Uuid::new_v4();
        let uuid_b = Uuid::new_v4();

        context.insert_instant_node(uuid_a);
        context.insert_instant_node(uuid_b);

        assert_eq!(context.instant_nodes, vec![uuid_a, uuid_b]);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn insert_instant_node_twice_panics() {
        let mut context = EffectGraphContext::new();
        let uuid = Uuid::new_v4();
        context.insert_instant_node(uuid);
        context.insert_instant_node(uuid);
    }

    #[test]
    fn replace_state_entities_rewrites_outputs_and_inputs() {
        let mut context = EffectGraphContext::new();
        let old = Entity::from_bits(1);
        let new = Entity::from_bits(10);

        let output_key = slot_pin(old.into(), "out");
        let input_key = slot_pin(old.into(), "in");
        context.insert_output_value(output_key, EffectValue::I32(5).into());
        context.insert_input_value(input_key, EffectValue::I32(6).into());

        let mut mapping = EntityHashMap::default();
        mapping.insert(old, new);
        context.replace_state_entities(mapping);

        let expected_output = slot_pin(new.into(), "out");
        let expected_input = slot_pin(new.into(), "in");
        assert_eq!(
            context.get_output_value(&expected_output),
            Some(&EffectNodeSlotValue::Value(EffectValue::I32(5)))
        );
        assert_eq!(
            context.get_input_value(&expected_input),
            Some(&EffectNodeSlotValue::Value(EffectValue::I32(6)))
        );
        assert_eq!(context.get_output_value(&output_key), None);
        assert_eq!(context.get_input_value(&input_key), None);
    }

    #[test]
    fn replace_state_entities_rewrites_connections() {
        let mut context = EffectGraphContext::new();
        let old_a = Entity::from_bits(1);
        let old_b = Entity::from_bits(2);
        let new_a = Entity::from_bits(10);
        let new_b = Entity::from_bits(20);

        context.add_exec_connection(
            exec_pin(old_a.into(), "finish"),
            &[exec_pin(old_b.into(), "start")],
        );
        context.add_slot_connection(
            slot_pin(old_a.into(), "out"),
            &[slot_pin(old_b.into(), "in")],
        );

        let mut mapping = EntityHashMap::default();
        mapping.insert(old_a, new_a);
        mapping.insert(old_b, new_b);
        context.replace_state_entities(mapping);

        let expected = vec![exec_pin(new_b.into(), "start")];
        assert_eq!(
            context.get_connected_output_exec_pins(&exec_pin(new_a.into(), "finish")),
            Some(&expected)
        );
        let expected_slots = vec![slot_pin(new_b.into(), "in")];
        assert_eq!(
            context.slot_connections.get(&slot_pin(new_a.into(), "out")),
            Some(&expected_slots)
        );
        assert_eq!(
            context.get_connected_output_exec_pins(&exec_pin(old_a.into(), "finish")),
            None
        );
        assert_eq!(
            context.slot_connections.get(&slot_pin(old_a.into(), "out")),
            None
        );
    }

    #[test]
    fn replace_state_entities_rewrites_entry_node_and_state_nodes() {
        let mut context = EffectGraphContext::new();
        let old = Entity::from_bits(1);
        let new = Entity::from_bits(10);

        context.set_entry_node(old);
        context.insert_state_node(old);

        let mut mapping = EntityHashMap::default();
        mapping.insert(old, new);
        context.replace_state_entities(mapping);

        assert_eq!(context.get_entry_node(), Some(new));
        assert_eq!(context.state_nodes, vec![new]);
    }

    #[test]
    fn replace_state_entities_keeps_uuid_entries() {
        let mut context = EffectGraphContext::new();
        let uuid = Uuid::new_v4();
        let uuid_key = slot_pin(EffectNodeId::Uuid(uuid), "out");
        context.insert_output_value(uuid_key, EffectValue::I32(9).into());

        let mut mapping = EntityHashMap::default();
        mapping.insert(Entity::from_bits(1), Entity::from_bits(2));
        context.replace_state_entities(mapping);

        assert_eq!(
            context.get_output_value(&uuid_key),
            Some(&EffectNodeSlotValue::Value(EffectValue::I32(9)))
        );
    }

    #[test]
    fn instant_node_map_insert_get_roundtrip() {
        let mut map = InstantEffectNodeMap::default();
        let node: Arc<dyn InstantEffectNode> = Arc::new(EffectNodeSeq::new());
        let uuid = node.get_uuid();

        assert!(map.get(uuid).is_none(), "未注册节点应返回 None");
        map.insert(uuid, node);

        let fetched = map.get(uuid).expect("注册后应能取回节点");
        assert_eq!(fetched.get_uuid(), uuid);
    }
}
