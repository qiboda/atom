pub mod ability_tag;
pub mod event;

use std::fmt::Debug;

use bevy::{
    prelude::{Children, Component, Entity, Query},
    reflect::Reflect,
};

use crate::graph::{context::EffectGraphContext, node::EffectNodeState};

#[derive(Debug, Default, Reflect, Copy, Clone, PartialEq)]
pub enum AbilityState {
    #[default]
    Uninitialized,
    Unactived,
    Actived,
}

/// add ability to entity
/// active ability
/// unactive ability
/// receive input
#[derive(Debug, Reflect, Component, Default, Clone)]
pub struct AbilityBase {
    state: AbilityState,
    graph: Option<Entity>,
}

impl AbilityBase {
    pub fn new(graph: Entity) -> Self {
        Self {
            state: AbilityState::Unactived,
            graph: Some(graph),
        }
    }

    pub fn get_state(&self) -> AbilityState {
        self.state
    }

    pub fn set_state(&mut self, state: AbilityState) {
        self.state = state;
    }

    pub fn get_graph(&self) -> Option<Entity> {
        self.graph
    }

    pub fn set_graph(&mut self, graph: Entity) {
        self.graph = Some(graph);
    }
}

/// set active from ability start, so set unactived when all children finished.
pub fn reset_graph_node_state(
    mut ability_query: Query<&mut AbilityBase>,
    graph_query: Query<(&EffectGraphContext, &Children)>,
    mut children_query: Query<&mut EffectNodeState>,
) {
    for mut ability_base in ability_query.iter_mut() {
        if ability_base.get_state() == AbilityState::Actived {
            if let Some(graph_entity) = ability_base.get_graph() {
                let (_context, children) = graph_query
                    .get(graph_entity)
                    .expect("ability ref a invalid graph");
                let all_finished = children
                    .iter()
                    .all(|child| children_query.get(*child).unwrap() == &EffectNodeState::Finished);

                if all_finished {
                    children.iter().for_each(|child| {
                        *children_query.get_mut(*child).unwrap() = EffectNodeState::Idle
                    });

                    ability_base.set_state(AbilityState::Unactived);
                }
            }
        }
    }
}
