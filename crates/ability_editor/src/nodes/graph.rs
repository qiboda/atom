use bevy::{
    prelude::*,
    utils::HashMap, ecs::system::EntityCommand,
};

use super::blackboard::EffectValue;

/// all children node  is graph nodes.
pub trait EffectGraph {}

pub trait EffectGraphBuilder {
    fn build(&self, commands: &mut Commands);
}

#[derive(Debug, Component)]
pub struct EffectGraphContext {
    pub blackboard: HashMap<Name, EffectValue>,
}
