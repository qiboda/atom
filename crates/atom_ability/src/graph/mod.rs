//! Effect Graph（技能图）核心模块。
//!
//! 技能图由节点（[`node`]）与引脚（[`pin`]）组成，节点间通过 Blackboard
//! （[`blackboard`]）共享数据，由执行器（[`executor`]）驱动，状态由
//! [`state`] 管理。图的构建入口见 [`builder`]，运行时实例与上下文见
//! [`context`]、[`graph_map`]，对外事件见 [`event`]。

use bevy::{app::App, prelude::*, reflect::Reflect};
use context::{EffectGraphContext, GraphRef, InstantEffectNodeMap};
use event::{
    trigger_clone_effect_graph_end, trigger_clone_effect_graph_start, trigger_effect_graph_add,
    trigger_effect_graph_exec, trigger_effect_graph_tickable, trigger_effect_graph_to_remove,
};
use executor::EffectGraphExecutorPlugin;
use graph_map::{EffectGraphBuilderMap, EffectGraphMap};
use state::{EffectGraphState, EffectGraphTickState, update_to_despawn_effect_graph};

use self::state::reset_effect_graph_state;

pub mod blackboard;
pub mod builder;
pub mod bundle;
pub mod context;
pub mod event;
pub mod executor;
pub mod graph_map;
pub mod node;
pub mod pin;
pub mod state;

/// Effect Graph 子系统插件。
///
/// 注册图执行器、调度集（Execute → UpdateNode → UpdateState 链式）、
/// 资源与类型反射，并挂载图生命周期相关的 observer（克隆/添加/执行/移除）。
#[derive(Debug, Default)]
pub struct EffectGraphPlugin;

impl Plugin for EffectGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectGraphExecutorPlugin)
            .configure_sets(
                Update,
                (
                    EffectGraphUpdateSystems::Execute,
                    EffectGraphUpdateSystems::UpdateNode,
                    EffectGraphUpdateSystems::UpdateState,
                )
                    .chain(),
            )
            .register_type::<EffectGraphContext>()
            .register_type::<EffectGraphState>()
            .register_type::<EffectGraphTickState>()
            .register_type::<GraphRef>()
            .init_resource::<InstantEffectNodeMap>()
            .init_resource::<EffectGraphMap>()
            .init_resource::<EffectGraphBuilderMap>()
            // .add_event::<CloneEffectGraphStartEvent>()
            // .add_event::<CloneEffectGraphEndEvent>()
            // .add_event::<EffectGraphAddEvent>()
            // .add_event::<EffectGraphExecEvent>()
            // .add_event::<EffectGraphRemoveEvent>()
            // .add_event::<EffectGraphTickableEvent>()
            .add_systems(
                Update,
                reset_effect_graph_state.in_set(EffectGraphUpdateSystems::UpdateState),
            )
            .add_systems(Last, update_to_despawn_effect_graph)
            .add_observer(trigger_clone_effect_graph_start)
            .add_observer(trigger_clone_effect_graph_end)
            .add_observer(trigger_effect_graph_add)
            .add_observer(trigger_effect_graph_exec)
            .add_observer(trigger_effect_graph_tickable)
            .add_observer(trigger_effect_graph_to_remove);
    }
}

/// 标记组件：标识实体拥有一个 Effect Graph。
///
/// 挂载在技能图根实体上，用于查询定位图实例。
#[derive(Debug, Default, Component, Reflect, Clone, Copy)]
#[reflect(Component)]
pub struct EffectGraphOwner;

/// Effect Graph 更新调度集。
///
/// 三个集合按顺序链式执行：`Execute`（执行节点）→ `UpdateNode`（节点状态更新）
/// → `UpdateState`（图状态收尾）。
#[derive(SystemSet, Debug, Default, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum EffectGraphUpdateSystems {
    /// 执行图内节点。
    #[default]
    Execute,
    /// 更新节点状态。
    UpdateNode,
    /// 更新图状态（含状态重置）。
    UpdateState,
}
