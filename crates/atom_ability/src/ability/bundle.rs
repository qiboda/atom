//! 技能场景模板：BSN（Bevy Scene Notation）构造技能实体与技能持有者实体。

use atom_datatables::effect::TbAbilityRow;
use bevy::prelude::*;

use crate::{
    attribute::attribute_set::AttributeSet,
    graph::EffectGraphOwner,
    stateset::{StateLayerTagContainer, StateLayerTagRegistry},
};

use super::{
    comp::{Ability, AbilityExecuteState, AbilityTickState},
    layertag::bundle::{build_ability_abort_tags, build_ability_start_tags},
};

/// 依据数据表行与状态层标签注册表构造技能实体场景。
///
/// 与迁移前 `AbilityBundle::new` 产物一致：技能标记 + 默认执行/节流状态 +
/// 数据表行 + Effect Graph 拥有者标记 + 6 个状态层标签容器。
pub fn spawn_ability(
    ability_row: TbAbilityRow,
    state_registry: &Res<StateLayerTagRegistry>,
) -> impl Scene {
    let data = ability_row.data();
    let (start_required, start_disable, added, removed) = build_ability_start_tags(
        &data.start_required_layertags,
        &data.start_disabled_layertags,
        &data.start_added_layertags,
        &data.start_removed_layertags,
        state_registry,
    );
    let (abort_required, abort_disable) = build_ability_abort_tags(
        &data.abort_required_layertags,
        &data.abort_disabled_layertags,
        state_registry,
    );

    bsn! {
        Ability
        AbilityExecuteState
        AbilityTickState
        template_value(ability_row)
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
