//! Buff 场景模板：BSN（Bevy Scene Notation）构造 buff 实体。

use atom_datatables::effect::TbBuffRow;
use bevy::prelude::*;

use crate::{graph::EffectGraphOwner, stateset::StateLayerTagRegistry};

use super::{
    layer::BuffLayer,
    layertag::bundle::{build_buff_abort_tags, build_buff_start_tags},
    state::{Buff, BuffExecuteState, BuffTickState},
    timer::BuffTime,
};

/// 依据数据表行与状态层标签注册表构造 buff 实体场景。
///
/// 与迁移前 `BuffBundle::new` 产物一致：buff 标记 + 默认执行/节流状态 +
/// 计时/层数 + 数据表行 + Effect Graph 拥有者标记 + 6 个状态层标签容器。
pub fn spawn_buff(buff_row: TbBuffRow, state_registry: &Res<StateLayerTagRegistry>) -> impl Scene {
    let data = buff_row.data();
    let (start_required, start_disable, added, removed) = build_buff_start_tags(
        &data.start_required_layertags,
        &data.start_disabled_layertags,
        &data.start_added_layertags,
        &data.start_removed_layertags,
        state_registry,
    );
    let (abort_required, abort_disable) = build_buff_abort_tags(
        &data.abort_required_layertags,
        &data.abort_disabled_layertags,
        state_registry,
    );

    // 迁移自 BuffBundle::new：计时与层数由数据表行驱动。
    let buff_time = BuffTime::new(
        data.duration,
        if data.interval > 0.0 {
            Some(data.interval)
        } else {
            None
        },
    );
    let buff_layer = BuffLayer::new(data.max_layer);

    bsn! {
        Buff
        EffectGraphOwner
        BuffExecuteState
        BuffTickState
        template_value(buff_time)
        template_value(buff_layer)
        template_value(buff_row)
        template_value(start_required)
        template_value(start_disable)
        template_value(added)
        template_value(removed)
        template_value(abort_required)
        template_value(abort_disable)
    }
}
