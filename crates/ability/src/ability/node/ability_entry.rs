use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::graph::node::plugin::TypedComponentIds;
use crate::graph::node::EffectNode;
use crate::{graph::node::StateEffectNode, impl_effect_node_pin_group};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeAbilityEntryPlugin;

impl Plugin for EffectNodeAbilityEntryPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        let component_id = world.init_component::<EffectNodeAbilityEntry>();
        let mut component_ids = world
            .get_resource_mut::<TypedComponentIds>()
            .expect("EffectNodePlugin should be added before this plugin");
        component_ids.insert::<EffectNodeAbilityEntry>(component_id);

        app.register_type::<EffectNodeAbilityEntry>();
    }
}

///////////////////////// Node Component /////////////////////////

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
