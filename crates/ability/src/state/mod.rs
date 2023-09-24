use std::ops::{Deref, Not};

use bevy::prelude::Component;
use layertag::layertag::LayerTag;


#[derive(Debug, Default, Component)]
pub struct StateTagContainer {
    states: Vec<Box<dyn LayerTag>>
}

impl StateTagContainer {
    pub fn add_state(&mut self, state: Box<dyn LayerTag>) {
        self.states.push(state);
    }

    pub fn remove_state(&mut self, state: Box<dyn LayerTag>) {
        self.states.retain(|x| x.deref().exact_match(state.deref()).not());
    }

    pub fn exist_state(&self, state: &dyn LayerTag) -> bool {
        self.states.iter().any(|x| state.exact_match((*x).deref()))
    }
}