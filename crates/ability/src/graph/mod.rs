use bevy::prelude::{Added, App, Commands, Component, Entity, First, Last, Plugin, Query};

use self::{
    builder::EffectGraphBuilder, context::EffectGraphContext, event::EffectEvent,
    state::update_to_remove,
};

pub mod base;
pub mod blackboard;
pub mod builder;
pub mod bundle;
pub mod context;
pub mod event;
pub mod node;
pub mod state;

#[derive(Debug, Default)]
pub struct EffectGraphPlugin {}

impl Plugin for EffectGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EffectEvent>()
            .add_systems(Last, update_to_remove);
    }
}

fn build_graph<T: EffectGraphBuilder + Component>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EffectGraphContext, &T), Added<T>>,
) {
    for (entity, mut graph_context, graph) in query.iter_mut() {
        graph.build(&mut commands, &mut graph_context, entity);
    }
}

#[derive(Default)]
pub struct EffectNodeGraphPlugin<T>(std::marker::PhantomData<T>);

impl<T: Component + EffectGraphBuilder> Plugin for EffectNodeGraphPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(First, build_graph::<T>);
    }
}
