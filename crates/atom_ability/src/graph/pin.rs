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
