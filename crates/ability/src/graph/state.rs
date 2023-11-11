use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query};
use bevy::log::info;

use super::{context::EffectGraphContext, node::EffectNodeExecuteState};

#[derive(Debug, Component, Default, PartialEq, Eq, Hash)]
pub enum EffectGraphState {
    #[default]
    Normal,
    ToRemove,
}

pub fn update_to_remove(
    mut commands: Commands,
    query: Query<(Entity, &EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (graph_entity, state, context) in query.iter() {
        if *state == EffectGraphState::ToRemove
            && context.nodes.iter().all(|node| {
                if let Ok(node_state) = node_state_query.get(*node) {
                    if *node_state == EffectNodeExecuteState::Actived {
                        return false;
                    }
                }
                true
            })
        {
            commands.entity(graph_entity).despawn_recursive();
            info!("despawn graph: {:?}", graph_entity);
        }
    }
}
