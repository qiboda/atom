//! 技能场景模板：BSN（Bevy Scene Notation）构造技能实体与技能持有者实体。

use bevy::prelude::*;

use crate::{
    attribute::attribute_set::AttributeSet,
    config::AbilityConfig,
    graph::EffectGraphOwner,
    stateset::{StateLayerTagContainer, StateLayerTagRegistry},
};

use super::{
    comp::{Ability, AbilityExecuteState, AbilityTickState},
    layertag::bundle::{build_ability_abort_tags, build_ability_start_tags},
};

/// 技能配置数据组件：技能实体携带的图类别数据（observer 数据源，替代已删除的 `TbAbilityRow`）。
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct AbilityConfigData {
    /// Effect Graph 图类别（构建技能效果图模板用）。
    pub graph_class: String,
}

/// 依据配置与状态层标签注册表构造技能实体场景。
///
/// 与迁移前 `AbilityBundle::new` 产物一致：技能标记 + 默认执行/节流状态 +
/// 配置数据 + Effect Graph 拥有者标记 + 6 个状态层标签容器。
pub fn spawn_ability(
    config: &AbilityConfig,
    state_registry: &Res<StateLayerTagRegistry>,
) -> impl Scene {
    let (start_required, start_disable, added, removed) = build_ability_start_tags(
        &config.start_required_layertags,
        &config.start_disabled_layertags,
        &config.start_added_layertags,
        &config.start_removed_layertags,
        state_registry,
    );
    let (abort_required, abort_disable) = build_ability_abort_tags(
        &config.abort_required_layertags,
        &config.abort_disabled_layertags,
        state_registry,
    );

    bsn! {
        Ability
        AbilityExecuteState
        AbilityTickState
        template_value(AbilityConfigData {
            graph_class: config.graph_class.clone(),
        })
        EffectGraphOwner
        template_value(start_required)
        template_value(start_disable)
        template_value(added)
        template_value(removed)
        template_value(abort_required)
        template_value(abort_disable)
    }
}

/// 构造技能持有者场景：属性集 + 状态层标签容器。
///
/// `T` 需满足 `Clone + Default + Unpin` 以通过 BSN `FromTemplate` blanket 注入。
pub fn spawn_ability_owner<T: AttributeSet + Component + Default + Clone + Unpin>() -> impl Scene {
    bsn! {
        T
        StateLayerTagContainer
    }
}
