use bevy::{ecs::system::EntityCommands, prelude::*};
use datatables::effect::TbBuffRow;

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

#[derive(Bundle, Reflect, Default)]
#[reflect(BuffBundleTrait)]
pub struct BuffBundle {
    pub buff: Buff,
    pub effect_graph_owner: EffectGraphOwner,
    pub execute_state: BuffExecuteState,
    pub tick_state: BuffTickState,
    pub buff_time: BuffTime,
    pub buff_layer: BuffLayer,
    pub buff_row: TbBuffRow,
    pub start_tag_bundle: BuffStartTagBundle,
    pub abort_tag_bundle: BuffAbortTagBundle,
}

impl BundleTrait for BuffBundle {
    fn spawn_bundle<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn(self)
    }
}

impl BuffBundleTrait for BuffBundle {}

impl BuffBundle {
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
