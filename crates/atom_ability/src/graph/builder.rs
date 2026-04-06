use std::fmt::Debug;

use bevy::prelude::{Commands, Entity, ResMut};

use super::context::InstantEffectNodeMap;

/// all children node is graph nodes.
pub trait EffectGraph {}

pub trait EffectGraphBuilder: Debug + Sync + Send {
    // TODO: move to another trait
    fn get_effect_graph_name(&self) -> &'static str;

    fn build(
        &self,
        commands: &mut Commands,
        instant_map: &mut ResMut<InstantEffectNodeMap>,
    ) -> Entity;
}
