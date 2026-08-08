//! 节点组件包与即时节点基础结构。

use super::{EffectNodeExecuteState, EffectNodeId, StateEffectNode};

use bevy::prelude::*;
use uuid::Uuid;

/// 状态节点组件包：状态节点类型 + 执行状态 + 节点 ID。
#[derive(Debug, Default, Bundle)]
pub struct StateEffectNodeBundle<T: Component + StateEffectNode> {
    /// 状态节点组件。
    pub state_node: T,
    /// 节点执行状态。
    pub execute_state: EffectNodeExecuteState,
    /// 节点 ID（实体形式）。
    pub node_id: EffectNodeId,
}

/// 即时节点基础结构：为即时节点提供 UUID 标识。
#[derive(Debug, Default, Reflect)]
pub struct InstantEffectNodeBase {
    /// 节点 UUID。
    pub node_id: Uuid,
}

impl InstantEffectNodeBase {
    /// 创建带新 UUID 的基础结构。
    pub fn new() -> Self {
        Self {
            node_id: Uuid::new_v4(),
        }
    }
}
