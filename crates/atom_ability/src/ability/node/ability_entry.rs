//! 技能入口状态节点：作为技能 Effect Graph 的起点（ready/start/abort）。

use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::graph::node::EffectNode;
use crate::graph::node::plugin::TypedComponentIds;
use crate::{graph::node::StateEffectNode, impl_effect_node_pin_group};

///////////////////////// Plugin /////////////////////////

/// 技能入口节点插件：向 [`TypedComponentIds`] 登记节点组件并注册类型反射。
#[derive(Debug, Default)]
pub struct EffectNodeAbilityEntryPlugin;

impl Plugin for EffectNodeAbilityEntryPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        let component_id = world
            .component_id::<EffectNodeAbilityEntry>()
            .expect("Component ID for EffectNodeAbilityEntry not found");
        let mut component_ids = world
            .get_resource_mut::<TypedComponentIds>()
            .expect("EffectNodePlugin should be added before this plugin");
        component_ids.insert::<EffectNodeAbilityEntry>(component_id);

        app.register_type::<EffectNodeAbilityEntry>();
    }
}

///////////////////////// Node Component /////////////////////////

/// 技能入口状态节点：提供 ready / start / abort 三个输出执行口，
/// 作为技能图的执行入口。
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectNodeAbilityEntry;

impl_effect_node_pin_group!(EffectNodeAbilityEntry,
    output => (
        ready => (),
        start => (),
        abort => ()
    )
);

impl EffectNode for EffectNodeAbilityEntry {}

impl StateEffectNode for EffectNodeAbilityEntry {}
