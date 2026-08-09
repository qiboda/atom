//! Buff 场景模板：BSN（Bevy Scene Notation）构造 buff 实体。

use bevy::prelude::*;

use crate::{config::BuffConfig, graph::EffectGraphOwner, stateset::StateLayerTagRegistry};

use super::{
    layer::BuffLayer,
    layertag::bundle::{build_buff_abort_tags, build_buff_start_tags},
    state::{Buff, BuffExecuteState, BuffTickState},
    timer::BuffTime,
};

/// Buff 配置数据组件：buff 实体携带的图类别数据（observer 数据源，替代已删除的 `TbBuffRow`）。
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct BuffConfigData {
    /// Effect Graph 图类别（构建 buff 效果图模板用）。
    pub graph_class: String,
}

/// 依据数据表行与状态层标签注册表构造 buff 实体场景。
///
/// 与迁移前 `BuffBundle::new` 产物一致：buff 标记 + 默认执行/节流状态 +
/// 计时/层数 + 配置数据 + Effect Graph 拥有者标记 + 6 个状态层标签容器。
pub fn spawn_buff(config: &BuffConfig, state_registry: &Res<StateLayerTagRegistry>) -> impl Scene {
    let (start_required, start_disable, added, removed) = build_buff_start_tags(
        &config.start_required_layertags,
        &config.start_disabled_layertags,
        &config.start_added_layertags,
        &config.start_removed_layertags,
        state_registry,
    );
    let (abort_required, abort_disable) = build_buff_abort_tags(
        &config.abort_required_layertags,
        &config.abort_disabled_layertags,
        state_registry,
    );

    // 迁移自 BuffBundle::new：计时与层数由配置驱动。
    let buff_time = BuffTime::new(
        config.duration,
        if config.interval > 0.0 {
            Some(config.interval)
        } else {
            None
        },
    );
    let buff_layer = BuffLayer::new(config.max_layer);

    bsn! {
        Buff
        EffectGraphOwner
        BuffExecuteState
        BuffTickState
        template_value(buff_time)
        template_value(buff_layer)
        template_value(BuffConfigData {
            graph_class: config.graph_class.clone(),
        })
        template_value(start_required)
        template_value(start_disable)
        template_value(added)
        template_value(removed)
        template_value(abort_required)
        template_value(abort_disable)
    }
}
