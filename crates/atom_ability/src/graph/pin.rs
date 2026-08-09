//! 引脚（Pin）类型：图内节点之间连线（exec 流与数据流）的端点标识。

use bevy::prelude::*;

use super::{
    blackboard::EffectValue,
    node::{
        EffectNodeId,
        pin::{EffectNodeExec, EffectNodeSlot},
    },
};

/// 执行流引脚：指向某节点的某个执行口（[`EffectNodeExec`]）。
///
/// 用于连接"上一个节点执行完 → 下一个节点执行"的先后关系。
#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Hash)]
pub struct EffectNodeExecPin {
    /// 目标节点 ID。
    pub node_id: EffectNodeId,
    /// 目标节点的执行口。
    pub exec: EffectNodeExec,
}

/// 数据流引脚：指向某节点的某个数据槽（[`EffectNodeSlot`]）。
///
/// 用于在节点之间传递数据值（通过 [`EffectNodeSlotValue::Ref`] 引用）。
#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Hash)]
pub struct EffectNodeSlotPin {
    /// 目标节点 ID。
    pub node_id: EffectNodeId,
    /// 目标节点的数据槽。
    pub slot: EffectNodeSlot,
}

/// 节点数据槽的值：直接持有值，或引用另一个节点的槽位。
#[derive(Debug, PartialEq, Clone, Reflect)]
pub enum EffectNodeSlotValue {
    /// 直接持有数据值。
    Value(EffectValue),
    /// 引用另一节点的槽位，运行时从该槽取值。
    Ref(EffectNodeSlotPin),
}

impl From<EffectValue> for EffectNodeSlotValue {
    fn from(value: EffectValue) -> Self {
        Self::Value(value)
    }
}

impl From<EffectNodeSlotPin> for EffectNodeSlotValue {
    fn from(value: EffectNodeSlotPin) -> Self {
        Self::Ref(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn slot_pin(entity: Entity, name: &'static str) -> EffectNodeSlotPin {
        EffectNodeSlotPin {
            node_id: EffectNodeId::Entity(entity),
            slot: EffectNodeSlot::new::<f32>(name),
        }
    }

    fn exec_pin(entity: Entity, name: &'static str) -> EffectNodeExecPin {
        EffectNodeExecPin {
            node_id: EffectNodeId::Entity(entity),
            exec: EffectNodeExec { name },
        }
    }

    #[test]
    fn exec_pin_equality_and_hash_roundtrip() {
        let a = exec_pin(Entity::from_bits(1), "finish");
        let b = exec_pin(Entity::from_bits(1), "finish");
        let c = exec_pin(Entity::from_bits(2), "finish");
        let d = exec_pin(Entity::from_bits(1), "start");

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b), "相等引脚应能通过 hash 命中");
        assert!(!set.contains(&c));
    }

    #[test]
    fn slot_pin_equality_and_hash_roundtrip() {
        let a = slot_pin(Entity::from_bits(1), "damage");
        let b = slot_pin(Entity::from_bits(1), "damage");
        let c = slot_pin(Entity::from_bits(2), "damage");
        let d = slot_pin(Entity::from_bits(1), "heal");

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);

        let mut map = HashMap::new();
        map.insert(a, 42);
        assert_eq!(map.get(&b), Some(&42));
        assert_eq!(map.get(&c), None);
    }

    #[test]
    fn exec_from_static_str() {
        let exec = EffectNodeExec::from("start");
        assert_eq!(exec.name, "start");
    }

    #[test]
    fn slot_new_records_type_id() {
        let slot = EffectNodeSlot::new::<i32>("count");
        assert_eq!(slot.name, "count");
        assert_eq!(slot.pin_type, std::any::TypeId::of::<i32>());
        assert_ne!(slot.pin_type, std::any::TypeId::of::<f32>());
    }

    #[test]
    fn slot_value_from_effect_value_is_value_variant() {
        let slot_value = EffectNodeSlotValue::from(EffectValue::I32(7));
        assert_eq!(slot_value, EffectNodeSlotValue::Value(EffectValue::I32(7)));
    }

    #[test]
    fn slot_value_from_slot_pin_is_ref_variant() {
        let pin = slot_pin(Entity::from_bits(3), "source");
        let slot_value = EffectNodeSlotValue::from(pin);
        assert_eq!(slot_value, EffectNodeSlotValue::Ref(pin));
    }

    #[test]
    fn slot_value_clone_preserves_value_and_ref() {
        let value = EffectNodeSlotValue::Value(EffectValue::F32(1.5));
        assert_eq!(value.clone(), value);

        let pin = slot_pin(Entity::from_bits(3), "source");
        let reference = EffectNodeSlotValue::Ref(pin);
        assert_eq!(reference.clone(), reference);
    }

    #[test]
    fn slot_value_value_and_ref_are_distinct() {
        let pin = slot_pin(Entity::from_bits(3), "source");
        assert_ne!(
            EffectNodeSlotValue::Value(EffectValue::I32(0)),
            EffectNodeSlotValue::Ref(pin)
        );
    }
}
