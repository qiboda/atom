use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::graph::node::plugin::TypedComponentIds;
use crate::graph::node::EffectNode;
use crate::{graph::node::StateEffectNode, impl_effect_node_pin_group};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeBuffEntryPlugin;

impl Plugin for EffectNodeBuffEntryPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        let component_id = world.component_id::<EffectNodeBuffEntry>().unwrap();
        let mut component_ids = world
            .get_resource_mut::<TypedComponentIds>()
            .expect("EffectNodePlugin should be added before this plugin");
        component_ids.insert::<EffectNodeBuffEntry>(component_id);

        app.register_type::<EffectNodeBuffEntry>();
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectNodeBuffEntry;

impl_effect_node_pin_group!(EffectNodeBuffEntry,
    output => (
        ready => (),
        start => (),
        looper => (),
        abort => (),
        end => (),
        add_layer => (added_layer: i32),
        remove_layer => (removed_layer: i32)
    )
);

impl EffectNode for EffectNodeBuffEntry {}

impl StateEffectNode for EffectNodeBuffEntry {}
