pub mod event;

use std::fmt::Debug;

use bevy::{
    prelude::{Component, Query},
    reflect::Reflect
};

use crate::graph::context::GraphRef;

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
    graph: Option<GraphRef>,
}

impl AbilityBase {
    pub fn new(graph: GraphRef) -> Self {
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

    pub fn get_graph(&self) -> Option<GraphRef> {
        self.graph
    }

    pub fn set_graph(&mut self, graph: GraphRef) {
        self.graph = Some(graph);
    }
}

/// set active from ability start, so set unactived when all children finished.
pub fn reset_graph_node_state(mut ability_query: Query<&mut AbilityBase>) {
    for mut ability_base in ability_query.iter_mut() {
        if ability_base.get_state() == AbilityState::Actived {
            ability_base.set_state(AbilityState::Unactived);
        }
    }
}
