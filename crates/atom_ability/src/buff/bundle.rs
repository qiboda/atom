//! Buff 组件包：buff 实体的完整组件集合。

use atom_datatables::effect::TbBuffRow;
use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    bundle::{BuffBundleTrait, BundleTrait, ReflectBuffBundleTrait},
    graph::EffectGraphOwner,
    stateset::StateLayerTagRegistry,
};

use super::{
    layer::BuffLayer,
    layertag::bundle::{BuffAbortTagBundle, BuffStartTagBundle},
    state::{Buff, BuffExecuteState, BuffTickState},
    timer::BuffTime,
};

/// Buff 实体组件包：标记、图拥有者、状态、计时、层数与状态层标签。
#[derive(Bundle, Reflect, Default)]
#[reflect(BuffBundleTrait)]
pub struct BuffBundle {
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
    /// Buff 数据表行。
    pub buff_row: TbBuffRow,
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
    /// 依据数据表行与状态层标签注册表构造 buff 组件包。
    pub fn new(buff_row: TbBuffRow, state_registry: &Res<StateLayerTagRegistry>) -> Self {
        let data = buff_row.data();
        let start_tag_bundle = BuffStartTagBundle::new(
            &data.start_required_layertags,
            &data.start_disabled_layertags,
            &data.start_added_layertags,
            &data.start_removed_layertags,
            state_registry,
        );

        let abort_tag_bundle = BuffAbortTagBundle::new(
            &data.abort_required_layertags,
            &data.abort_disabled_layertags,
            state_registry,
        );

        let buff_time = BuffTime::new(
            data.duration,
            if data.interval > 0.0 {
                Some(data.interval)
            } else {
                None
            },
        );
        let buff_layer = BuffLayer::new(data.max_layer);

        Self {
            buff_row,
            start_tag_bundle,
            abort_tag_bundle,
            buff_time,
            buff_layer,
            ..Default::default()
        }
    }
}
