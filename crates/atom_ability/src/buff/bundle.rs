//! Buff 组件包：buff 实体的完整组件集合。

use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    bundle::{BuffBundleTrait, BundleTrait, ReflectBuffBundleTrait},
    config::BuffConfig,
    graph::EffectGraphOwner,
    stateset::StateLayerTagRegistry,
};

use super::{
    layer::BuffLayer,
    layertag::bundle::{BuffAbortTagBundle, BuffStartTagBundle},
    state::{Buff, BuffExecuteState, BuffTickState},
    timer::BuffTime,
};

/// Buff 配置数据组件：buff 实体携带的图类别数据（observer 数据源，替代已删除的 `TbBuffRow`）。
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct BuffConfigData {
    /// Effect Graph 图类别（构建 buff 效果图模板用）。
    pub graph_class: String,
}

/// Buff 实体组件包：标记、图拥有者、状态、计时、层数、配置数据与状态层标签。
///
/// 字段顺序有约束：`config_data` 必须在 `buff` **之前**——`On<Add, Buff>` observer
/// 在 bundle 插入过程中触发（按字段序逐组件插入），此时后插入的组件尚不在实体 archetype
/// 中（`QueryDoesNotMatch`），observer 按新数据形态查询会落空（RED 测试实证）。
#[derive(Bundle, Reflect, Default)]
#[reflect(BuffBundleTrait)]
pub struct BuffBundle {
    /// Buff 配置数据（observer 数据源，替代已删除的 `TbBuffRow`；须先于 `buff` 插入）。
    pub config_data: BuffConfigData,
    /// Buff 标记组件。
    pub buff: Buff,
    /// Effect Graph 拥有者标记。
    pub effect_graph_owner: EffectGraphOwner,
    /// Buff 执行状态。
    pub execute_state: BuffExecuteState,
    /// Buff 节流状态。
    pub tick_state: BuffTickState,
    /// Buff 计时器。
    pub buff_time: BuffTime,
    /// Buff 层数。
    pub buff_layer: BuffLayer,
    /// 开始阶段状态层标签包。
    pub start_tag_bundle: BuffStartTagBundle,
    /// 中断阶段状态层标签包。
    pub abort_tag_bundle: BuffAbortTagBundle,
}

impl BundleTrait for BuffBundle {
    fn spawn_bundle<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self)
    }
}

impl BuffBundleTrait for BuffBundle {}

impl BuffBundle {
    /// 依据配置数据与状态层标签注册表构造 buff 组件包。
    pub fn new(config: &BuffConfig, state_registry: &StateLayerTagRegistry) -> Self {
        let start_tag_bundle = BuffStartTagBundle::new(
            &config.start_required_layertags,
            &config.start_disabled_layertags,
            &config.start_added_layertags,
            &config.start_removed_layertags,
            state_registry,
        );

        let abort_tag_bundle = BuffAbortTagBundle::new(
            &config.abort_required_layertags,
            &config.abort_disabled_layertags,
            state_registry,
        );

        let buff_time = BuffTime::new(
            config.duration,
            if config.interval > 0.0 {
                Some(config.interval)
            } else {
                None
            },
        );
        let buff_layer = BuffLayer::new(config.max_layer);

        Self {
            config_data: BuffConfigData {
                graph_class: config.graph_class.clone(),
            },
            start_tag_bundle,
            abort_tag_bundle,
            buff_time,
            buff_layer,
            ..Default::default()
        }
    }
}
