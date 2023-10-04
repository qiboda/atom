use bevy::prelude::*;

use crate::graph::{
    context::EffectGraphContext,
    event::{EffectNodeEvent, EffectNodeStartEvent},
    state::EffectGraphState,
};

use super::graph_map::EffectGraphMap;

/// add ability to entity
/// active ability
/// unactive ability
/// receive input
#[derive(Debug, Component, Default, Reflect, Copy, Clone, PartialEq)]
pub enum EffectState {
    #[default]
    Unactived,
    CheckCanActive,
    ActiveBefore,
    Actived,
    BeforeUnactived,
}

/// set active from ability start, so set unactived when all children finished.
pub fn update_to_unactive_state(mut effect_query: Query<&mut EffectState>) {
    for mut effect_state in effect_query.iter_mut() {
        match *effect_state {
            EffectState::Unactived => {}
            EffectState::CheckCanActive => {}
            EffectState::ActiveBefore => {}
            EffectState::Actived => {
                *effect_state = EffectState::BeforeUnactived;
            }
            EffectState::BeforeUnactived => {
                *effect_state = EffectState::Unactived;
            }
        }
    }
}

pub fn update_to_active_state(
    mut state_query: Query<(Entity, &mut EffectState)>,
    graph_query: Query<&EffectGraphContext>,
    effect_graph_map: Res<EffectGraphMap>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
) {
    for (entity, mut state) in state_query.iter_mut() {
        match *state {
            EffectState::Unactived => {}
            EffectState::CheckCanActive => {
                *state = EffectState::ActiveBefore;
            }
            EffectState::ActiveBefore => {
                *state = EffectState::Actived;

                let graph = effect_graph_map.map.get(&entity).unwrap();
                let graph_context = graph_query.get(graph.get_entity()).unwrap();
                if let Some(entry_node) = graph_context.entry_node {
                    event_writer.send(EffectNodeStartEvent::new(entry_node));
                }
            }
            EffectState::Actived => {}
            EffectState::BeforeUnactived => {}
        }
    }
}

pub fn on_remove_effect(
    mut removed_ability: RemovedComponents<EffectState>,
    mut effect_graph_map: ResMut<EffectGraphMap>,
    mut query: Query<&mut EffectGraphState>,
) {
    for ability in removed_ability.iter() {
        if let Some(graph_ref) = effect_graph_map.map.remove(&ability) {
            let mut graph_state = query.get_mut(graph_ref.get_entity()).unwrap();
            *graph_state = EffectGraphState::ToRemove;
        }
    }
}
