use bevy::{
    ecs::component::Mutable,
    prelude::{App, Component, MessageReader, Plugin, Query},
};

use self::{
    event::EffectEvent,
    node::{EffectNode, EffectNodeState},
};

pub mod blackboard;
pub mod bundle;
pub mod event;
pub mod graph;
pub mod node;
pub mod pin;

#[derive(Debug)]
pub struct EffectGraphPlugin {}

impl Plugin for EffectGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EffectEvent>();
    }
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_event<T: EffectNode + Component<Mutability = Mutable>>(
    mut query: Query<(&mut T, &mut EffectNodeState)>,
    mut event: MessageReader<EffectEvent>,
) {
    for event in event.read() {
        match event {
            EffectEvent::Start(entity) => {
                if let Ok((mut node, mut state)) = query.get_mut(*entity)
                    && *state == EffectNodeState::Idle
                {
                    node.start();
                    *state = EffectNodeState::Running;
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
                if let Ok((mut node, mut state)) = query.get_mut(*entity)
                    && (*state == EffectNodeState::Running || *state == EffectNodeState::Paused)
                {
                    node.abort();
                    *state = EffectNodeState::Aborted;
                }
            }
        }
    }
}
