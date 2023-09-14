use bevy::{prelude::*, utils::HashMap};

use super::blackboard::EffectValue;

/// all children node  is graph nodes.
pub trait EffectGraph {}

pub trait EffectGraphBuilder {
    fn build(&self, commands: &mut Commands);
}

#[derive(Debug, Component, Reflect)]
pub struct EffectOutputKey {
    pub node: Entity,
    pub output_key: String,
}

#[derive(Debug, Component)]
pub struct EffectGraphContext {
    pub blackboard: HashMap<Name, EffectValue>,

    pub outputs: HashMap<EffectOutputKey, EffectValue>,
}

#[derive(Debug, Bundle)]
pub struct EffectGraphBundle<EffectGraphType: EffectGraph + Component> {
    pub context: EffectGraphContext,
    pub graph: EffectGraphType,
}
