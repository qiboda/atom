use bevy::prelude::{
    info, Added, App, Commands, Component, Entity, EventReader, First, Plugin, Query,
};

use self::{
    context::EffectGraphContext,
    event::EffectEvent,
    node::{EffectNode, EffectNodeState}, builder::EffectGraphBuilder,
};

pub mod base;
pub mod blackboard;
pub mod bundle;
pub mod context;
pub mod event;
pub mod node;
pub mod builder;

#[derive(Debug, Default)]
pub struct EffectGraphPlugin {}

impl Plugin for EffectGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EffectEvent>();
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

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_event<T: EffectNode + Component>(
    mut query: Query<(&mut T, &mut EffectNodeState)>,
    mut event: EventReader<EffectEvent>,
) {
    for event in event.iter() {
        match event {
            EffectEvent::Start(entity) => {
                if let Ok((mut node, mut state)) = query.get_mut(*entity) {
                    info!("node start: {:?}", entity);
                    if *state == EffectNodeState::Idle {
                        info!("node start ok: {:?}", entity);
                        node.start();
                        *state = EffectNodeState::Running;
                    }
                }
            }
            EffectEvent::Pause => {
                for (mut node, mut state) in query.iter_mut() {
                    node.pause();
                    *state = EffectNodeState::Paused;
                }
            }
            EffectEvent::Resume => {
                for (mut node, mut state) in query.iter_mut() {
                    node.resume();
                    *state = EffectNodeState::Running;
                }
            }
            EffectEvent::Abort(entity) => {
                if let Ok((mut node, mut state)) = query.get_mut(*entity) {
                    if *state == EffectNodeState::Running || *state == EffectNodeState::Paused {
                        node.abort();
                        *state = EffectNodeState::Aborted;
                    }
                }
            }
        }
    }
}
