use bevy::log::info;
use bevy::prelude::*;

use super::{context::EffectGraphContext, node::EffectNodeExecuteState};

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum EffectGraphTickState {
    #[default]
    Ticked,
    Paused,
}

#[derive(Debug, Component, Default, PartialEq, Eq, Hash, Reflect, Clone, Copy)]
#[reflect(Component)]
pub enum EffectGraphState {
    #[default]
    Inactive,
    Active,
    ToRemove,
}

pub fn reset_effect_graph_state(
    mut query: Query<(&mut EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (mut state, context) in query.iter_mut() {
        match *state {
            EffectGraphState::Inactive => {}
            EffectGraphState::Active => {
                if context.state_nodes.iter().all(|node| {
                    if let Ok(node_state) = node_state_query.get(*node) {
                        if *node_state == EffectNodeExecuteState::Idle {
                            return true;
                        }
                    }
                    false
                }) {
                    *state = EffectGraphState::Inactive;
                }
            }
            EffectGraphState::ToRemove => {}
        }
    }
}

pub fn update_to_despawn_effect_graph(
    mut commands: Commands,
    query: Query<(Entity, &EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (graph_entity, state, context) in query.iter() {
        if *state == EffectGraphState::ToRemove
            && context.state_nodes.iter().all(|node| {
                if let Ok(node_state) = node_state_query.get(*node) {
                    if *node_state == EffectNodeExecuteState::Idle {
                        return true;
                    }
                }
                false
            })
        {
            commands.entity(graph_entity).despawn_recursive();
            info!("despawn graph: {:?}", graph_entity);
        }
    }
}
