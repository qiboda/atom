pub mod event;

use bevy::{
    prelude::{
        App, Component, Entity, Last, Plugin, PostUpdate, Query, RemovedComponents, ResMut,
        Resource,
    },
    reflect::Reflect,
    utils::HashMap,
};

use crate::graph::{context::GraphRef, state::EffectGraphState};

#[derive(Debug, Default)]
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AbilityGraph::default())
            .add_systems(PostUpdate, reset_graph_node_state)
            .add_systems(Last, on_remove_ability);
    }
}

#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Ability;

/// add ability to entity
/// active ability
/// unactive ability
/// receive input
#[derive(Debug, Component, Default, Reflect, Copy, Clone, PartialEq)]
pub enum AbilityState {
    #[default]
    Unactived,
    Actived,
}

#[derive(Debug, Resource, Default, Clone)]
pub struct AbilityGraph {
    pub map: HashMap<Entity, GraphRef>,
}

/// set active from ability start, so set unactived when all children finished.
pub fn reset_graph_node_state(mut ability_query: Query<&mut AbilityState>) {
    for mut ability_base in ability_query.iter_mut() {
        if *ability_base == AbilityState::Actived {
            *ability_base = AbilityState::Unactived;
        }
    }
}

pub fn on_remove_ability(
    mut removed_ability: RemovedComponents<Ability>,
    mut ability_graph: ResMut<AbilityGraph>,
    mut query: Query<&mut EffectGraphState>,
) {
    for ability in removed_ability.iter() {
        if let Some(graph_ref) = ability_graph.map.remove(&ability) {
            let mut graph_state = query.get_mut(graph_ref.get_entity()).unwrap();
            *graph_state = EffectGraphState::ToRemove;
        }
    }
}
