use bevy::prelude::*;

use super::{
    context::EffectGraphContext,
    node::{EffectNode, EffectNodeState, EffectNodeUuid},
};

// #[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
// pub enum EffectEventSet {
//     ReceiveEvent,
//     HandleEvent,
//     FlushPending,
// }

#[derive(Debug, Default)]
pub struct EffectNodeEventPlugin<T: EffectNode + Component>(std::marker::PhantomData<T>);

impl<T: EffectNode + Component> Plugin for EffectNodeEventPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource::<EffectNodePending<T>>(EffectNodePending::<T>::default())
            .add_systems(
                PreUpdate,
                (
                    receive_effect_event::<T>,
                    handle_effect_event::<T>,
                    flush_pending::<T>,
                )
                    .chain(),
            );
    }
}

#[derive(Event)]
pub enum EffectEvent {
    Start(Entity),
    Abort(Entity),
    Pause(Entity),
    Resume(Entity),
}

/**
 * Pending effect node.
 * Every effect node Pendng this component.
 */
#[derive(Resource, Debug)]
pub struct EffectNodePending<T: EffectNode> {
    pub pending_start: Vec<Entity>,
    pub pending_pause: Vec<Entity>,
    pub pending_resume: Vec<Entity>,
    pub pending_abort: Vec<Entity>,
    pub marker: std::marker::PhantomData<T>,
}

impl<T: EffectNode> Default for EffectNodePending<T> {
    fn default() -> Self {
        Self {
            pending_start: Default::default(),
            pending_pause: Default::default(),
            pending_resume: Default::default(),
            pending_abort: Default::default(),
            marker: Default::default(),
        }
    }
}

/// flush in last
pub fn flush_pending<T: EffectNode + Component>(mut pending: ResMut<EffectNodePending<T>>) {
    pending.pending_start.clear();
    pending.pending_pause.clear();
    pending.pending_resume.clear();
    pending.pending_abort.clear();
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_event<T: EffectNode + Component>(
    mut pending: ResMut<EffectNodePending<T>>,
    mut event_reader: EventReader<EffectEvent>,
) {
    for event in event_reader.iter() {
        match event {
            EffectEvent::Start(entity) => {
                pending.pending_start.push(*entity);
            }
            EffectEvent::Pause(entity) => {
                pending.pending_pause.push(*entity);
            }
            EffectEvent::Resume(entity) => {
                pending.pending_resume.push(*entity);
            }
            EffectEvent::Abort(entity) => {
                pending.pending_abort.push(*entity);
            }
        }
    }
}

pub fn handle_effect_event<T: EffectNode + Component>(
    mut commands: Commands,
    mut query: Query<(&mut T, &EffectNodeUuid, &mut EffectNodeState, &Parent)>,
    pending: Res<EffectNodePending<T>>,
    mut graph_query: Query<&mut EffectGraphContext>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for start_entity in pending.pending_start.iter() {
        if let Ok((mut node, node_uuid, mut state, parent)) = query.get_mut(*start_entity) {
            if *state != EffectNodeState::Paused {
                info!(
                    "node {} start: {:?}",
                    std::any::type_name::<T>(),
                    start_entity
                );
                let mut graph_context = graph_query.get_mut(parent.get()).unwrap();
                node.start(
                    &mut commands,
                    *start_entity,
                    node_uuid,
                    &mut state,
                    &mut graph_context,
                    &mut event_writer,
                );
            }
        }
    }

    for pause_entity in pending.pending_pause.iter() {
        if let Ok((mut node, _node_uuid, state, parent)) = query.get_mut(*pause_entity) {
            if *state == EffectNodeState::Paused {
                continue;
            }
            info!(
                "node {} pause: {:?}",
                std::any::type_name::<T>(),
                pause_entity
            );
            let mut _graph_context = graph_query.get_mut(parent.get()).unwrap();
            node.pause(
                    // &mut commands,
                    // *pause_entity,
                    // node_uuid,
                    // &mut state,
                    // &mut graph_context,
                    // &mut event_writer,
                );
        }
    }

    for resume_entity in pending.pending_resume.iter() {
        if let Ok((mut node, _node_uuid, state, parent)) = query.get_mut(*resume_entity) {
            if *state == EffectNodeState::Paused {
                info!(
                    "node {} resume: {:?}",
                    std::any::type_name::<T>(),
                    resume_entity
                );
                let mut _graph_context = graph_query.get_mut(parent.get()).unwrap();
                node.resume(
                    // &mut commands,
                    // *resume_entity,
                    // node_uuid,
                    // &mut state,
                    // &mut graph_context,
                    // &mut event_writer,
                );
            }
        }
    }

    for abort_entity in pending.pending_abort.iter() {
        if let Ok((mut node, _node_uuid, mut _state, _parent)) = query.get_mut(*abort_entity) {
            info!(
                "node {} abort: {:?}",
                std::any::type_name::<T>(),
                abort_entity
            );
            // let mut graph_context = graph_query.get_mut(parent.get()).unwrap();
            node.abort(
                    // &mut commands,
                    // *abort_entity,
                    // node_uuid,
                    // &mut state,
                    // &mut graph_context,
                    // &mut event_writer,
                );
        }
    }
}
