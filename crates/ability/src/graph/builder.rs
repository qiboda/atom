use bevy::prelude::{Commands, Entity};

use super::context::EffectGraphContext;


/// all children node is graph nodes.
pub trait EffectGraph: EffectGraphBuilder {}

pub trait EffectGraphBuilder {
    fn build(
        &self,
        commands: &mut Commands,
        effect_graph_context: &mut EffectGraphContext,
        parent: Entity,
    );
}
