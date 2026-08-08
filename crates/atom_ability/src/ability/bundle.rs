//! 技能组件包：技能实体的完整组件集合。

use atom_datatables::effect::TbAbilityRow;
use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    attribute::attribute_set::AttributeSet,
    bundle::{AbilityBundleTrait, BundleTrait, ReflectAbilityBundleTrait},
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

/// 技能实体组件包：执行/节流状态、技能标记、数据表行与 Effect Graph 标记。
#[derive(Bundle, Reflect, Default)]
#[reflect(AbilityBundleTrait)]
pub struct AbilityBundle {
    /// 技能执行状态。
    pub execute_state: AbilityExecuteState,
    /// 技能节流状态。
    pub tick_state: AbilityTickState,
    /// 技能标记组件。
    pub ability: Ability,
    /// 技能数据表行。
    pub ability_row: TbAbilityRow,
    /// Effect Graph 拥有者标记。
    pub effect_graph_owner: EffectGraphOwner,
    /// 开始阶段状态层标签包。
    pub start_tag_bundle: AbilityStartTagBundle,
    /// 中断阶段状态层标签包。
    pub abort_tag_bundle: AbilityAbortTagBundle,
}

impl AbilityBundle {
    /// 依据数据表行与状态层标签注册表构造技能组件包。
    pub fn new(ability_row: TbAbilityRow, state_registry: &Res<StateLayerTagRegistry>) -> Self {
        let data = ability_row.data();
        let start_tag_bundle = AbilityStartTagBundle::new(
            &data.start_required_layertags,
            &data.start_disabled_layertags,
            &data.start_added_layertags,
            &data.start_removed_layertags,
            state_registry,
        );

        let abort_tag_bundle = AbilityAbortTagBundle::new(
            &data.abort_required_layertags,
            &data.abort_disabled_layertags,
            state_registry,
        );

        Self {
            ability_row,
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
