//! Effect 入口状态节点：作为 effect Effect Graph 的起点（ready/start/abort/end）。

use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::graph::node::EffectNode;
use crate::graph::node::plugin::TypedComponentIds;
use crate::{graph::node::StateEffectNode, impl_effect_node_pin_group};

///////////////////////// Plugin /////////////////////////

/// Effect 入口节点插件：向 [`TypedComponentIds`] 登记节点组件并注册类型反射。
#[derive(Debug, Default)]
pub struct EffectNodeEffectEntryPlugin;

impl Plugin for EffectNodeEffectEntryPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        let component_id = world.component_id::<EffectNodeEffectEntry>().expect(
            "EffectNodeEffectEntry component not found! You need registry EffectNodePlugin before",
        );
        let mut component_ids = world
            .get_resource_mut::<TypedComponentIds>()
            .expect("EffectNodePlugin should be added before this plugin");
        component_ids.insert::<EffectNodeEffectEntry>(component_id);

        app.register_type::<EffectNodeEffectEntry>();
    }
}

///////////////////////// Node Component /////////////////////////

/// Effect 入口状态节点：提供 ready / start / abort / end 输出执行口，作为 effect 图的执行入口。
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectNodeEffectEntry;

impl_effect_node_pin_group!(EffectNodeEffectEntry,
    output => (
        ready => (),
        start => (),
        abort => (),
        end => ()
    )
);

impl EffectNode for EffectNodeEffectEntry {}

impl StateEffectNode for EffectNodeEffectEntry {}
