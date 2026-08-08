//! 技能组件包：技能实体的完整组件集合。

use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    attribute::attribute_set::AttributeSet,
    bundle::{AbilityBundleTrait, BundleTrait, ReflectAbilityBundleTrait},
    config::AbilityConfig,
    graph::EffectGraphOwner,
    stateset::{StateLayerTagContainer, StateLayerTagRegistry},
};

use super::{
    comp::{Ability, AbilityExecuteState, AbilityTickState},
    layertag::bundle::{AbilityAbortTagBundle, AbilityStartTagBundle},
};

/// 技能持有者组件包：属性集 + 状态层标签容器。
#[derive(Bundle, Default)]
pub struct AbilityOwnerBundle<T: AttributeSet> {
    /// 技能属性集。
    pub attribute_set: T,
    /// 技能所属状态层标签容器。
    pub state_set: StateLayerTagContainer,
}

/// 技能配置数据组件：技能实体携带的图类别数据（observer 数据源，替代已删除的 `TbAbilityRow`）。
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct AbilityConfigData {
    /// Effect Graph 图类别（构建技能效果图模板用）。
    pub graph_class: String,
}

/// 技能实体组件包：执行/节流状态、技能标记、配置数据与 Effect Graph 标记。
///
/// 字段顺序有约束：`config_data` 必须在 `ability` **之前**——`On<Add, Ability>` observer
/// 在 bundle 插入过程中触发（按字段序逐组件插入），此时后插入的组件尚不在实体 archetype
/// 中（`QueryDoesNotMatch`），observer 按新数据形态查询会落空（RED 测试实证）。
#[derive(Bundle, Reflect, Default)]
#[reflect(AbilityBundleTrait)]
pub struct AbilityBundle {
    /// 技能执行状态。
    pub execute_state: AbilityExecuteState,
    /// 技能节流状态。
    pub tick_state: AbilityTickState,
    /// 技能配置数据（observer 数据源，替代已删除的 `TbAbilityRow`；须先于 `ability` 插入）。
    pub config_data: AbilityConfigData,
    /// 技能标记组件。
    pub ability: Ability,
    /// Effect Graph 拥有者标记。
    pub effect_graph_owner: EffectGraphOwner,
    /// 开始阶段状态层标签包。
    pub start_tag_bundle: AbilityStartTagBundle,
    /// 中断阶段状态层标签包。
    pub abort_tag_bundle: AbilityAbortTagBundle,
}

impl AbilityBundle {
    /// 依据配置数据与状态层标签注册表构造技能组件包。
    pub fn new(config: &AbilityConfig, state_registry: &StateLayerTagRegistry) -> Self {
        let start_tag_bundle = AbilityStartTagBundle::new(
            &config.start_required_layertags,
            &config.start_disabled_layertags,
            &config.start_added_layertags,
            &config.start_removed_layertags,
            state_registry,
        );

        let abort_tag_bundle = AbilityAbortTagBundle::new(
            &config.abort_required_layertags,
            &config.abort_disabled_layertags,
            state_registry,
        );

        Self {
            config_data: AbilityConfigData {
                graph_class: config.graph_class.clone(),
            },
            start_tag_bundle,
            abort_tag_bundle,
            ..Default::default()
        }
    }
}

impl BundleTrait for AbilityBundle {
    fn spawn_bundle<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self)
    }
}

impl AbilityBundleTrait for AbilityBundle {}
