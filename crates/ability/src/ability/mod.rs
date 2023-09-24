pub mod ability_tag;
pub mod event;

use std::fmt::Debug;

use bevy::{prelude::{Entity, Component}, reflect::Reflect};

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
