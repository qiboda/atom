//! Effect Graph 组件包：一次插入图实体的全部核心组件。

use bevy::prelude::{Bundle, Component};

use super::{
    builder::EffectGraph,
    context::EffectGraphContext,
    state::{EffectGraphState, EffectGraphTickState},
};

/// Effect Graph 实体的组件包：上下文 + 图状态 + 节流状态 + 图标记组件。
#[derive(Debug, Bundle, Default)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component + Default> {
    /// 图运行时上下文。
    pub context: EffectGraphContext,
    /// 图生命周期状态。
    pub state: EffectGraphState,
    /// 图节流状态。
    pub tick_state: EffectGraphTickState,
    /// 图标记组件（具体图类型）。
    pub graph: EffectGraphType,
}
