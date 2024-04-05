use std::ops::Not;

use bevy::prelude::*;

use super::node::{EffectNode, EffectNodeTickState};

// #[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
// pub enum EffectEventSet {
//     ReceiveEvent,
//     HandleEvent,
//     FlushPending,
// }

#[derive(Debug, Default)]
pub struct EffectNodeEventPlugin;

impl Plugin for EffectNodeEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EffectNodeCheckStartEvent>()
            .add_event::<EffectNodeStartEvent>()
            .add_event::<EffectNodeAbortEvent>()
            .add_event::<EffectNodePauseEvent>()
            .add_event::<EffectNodeResumeEvent>()
            .insert_resource::<EffectNodePendingEvents>(EffectNodePendingEvents::default())
            .add_systems(
                PreUpdate,
                (
                    receive_effect_node_check_start_event,
                    receive_effect_node_start_event,
                    receive_effect_node_abort_event,
                    receive_effect_node_pause_event,
                    receive_effect_node_resume_event,
                ),
            )
            .add_systems(Last, (flush_pending,).chain());
    }
}

pub trait EffectNodeEvent {
    fn new(node: Entity) -> Self;
}

#[derive(Event)]
pub struct EffectNodeCheckStartEvent {
    pub node: Entity,
}

impl EffectNodeEvent for EffectNodeCheckStartEvent {
    fn new(node: Entity) -> Self {
        Self { node }
    }
}

#[derive(Event)]
pub struct EffectNodeStartEvent {
    pub node: Entity,
}

impl EffectNodeEvent for EffectNodeStartEvent {
    fn new(node: Entity) -> Self {
        Self { node }
    }
}

#[derive(Event)]
pub struct EffectNodeAbortEvent {
    pub node: Entity,
}

impl EffectNodeEvent for EffectNodeAbortEvent {
    fn new(node: Entity) -> Self {
        Self { node }
    }
}

#[derive(Event)]
pub struct EffectNodePauseEvent {
    pub node: Entity,
}

impl EffectNodeEvent for EffectNodePauseEvent {
    fn new(node: Entity) -> Self {
        Self { node }
    }
}

#[derive(Event)]
pub struct EffectNodeResumeEvent {
    pub node: Entity,
}

impl EffectNodeEvent for EffectNodeResumeEvent {
    fn new(node: Entity) -> Self {
        Self { node }
    }
}

/**
 * Pending effect node.
 * Every effect node Pending this component.
 */
#[derive(Resource, Debug, Default, Reflect)]
pub struct EffectNodePendingEvents {
    pub pending_check_can_start: Vec<Entity>,
    pub pending_start: Vec<Entity>,
    pub pending_pause: Vec<Entity>,
    pub pending_resume: Vec<Entity>,
    pub pending_abort: Vec<Entity>,
}

pub fn node_can_check_start() -> impl Fn(Res<EffectNodePendingEvents>) -> bool {
    |pending: Res<EffectNodePendingEvents>| pending.pending_check_can_start.is_empty().not()
}

pub fn node_can_start() -> impl Fn(Res<EffectNodePendingEvents>) -> bool {
    |pending: Res<EffectNodePendingEvents>| pending.pending_start.is_empty().not()
}

pub fn node_can_abort() -> impl Fn(Res<EffectNodePendingEvents>) -> bool {
    |pending: Res<EffectNodePendingEvents>| pending.pending_abort.is_empty().not()
}

pub fn node_can_pause() -> impl Fn(Res<EffectNodePendingEvents>) -> bool {
    |pending: Res<EffectNodePendingEvents>| pending.pending_pause.is_empty().not()
}

pub fn node_can_resume() -> impl Fn(Res<EffectNodePendingEvents>) -> bool {
    |pending: Res<EffectNodePendingEvents>| pending.pending_resume.is_empty().not()
}

/// flush in last
pub fn flush_pending(mut pending: ResMut<EffectNodePendingEvents>) {
    pending.pending_check_can_start.clear();
    pending.pending_start.clear();
    pending.pending_pause.clear();
    pending.pending_resume.clear();
    pending.pending_abort.clear();
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_node_check_start_event(
    mut pending: ResMut<EffectNodePendingEvents>,
    mut event_reader: EventReader<EffectNodeCheckStartEvent>,
) {
    for event in event_reader.read() {
        pending.pending_check_can_start.push(event.node);
    }
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_node_start_event(
    mut pending: ResMut<EffectNodePendingEvents>,
    mut event_reader: EventReader<EffectNodeStartEvent>,
) {
    for event in event_reader.read() {
        pending.pending_start.push(event.node);
    }
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_node_abort_event(
    mut pending: ResMut<EffectNodePendingEvents>,
    mut event_reader: EventReader<EffectNodeAbortEvent>,
) {
    for event in event_reader.read() {
        pending.pending_abort.push(event.node);
    }
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_node_pause_event(
    mut pending: ResMut<EffectNodePendingEvents>,
    mut event_reader: EventReader<EffectNodePauseEvent>,
) {
    for event in event_reader.read() {
        pending.pending_pause.push(event.node);
    }
}

/**
 * Receive effect event.
 * Every effect node should add this system in PreUpdate Stage.
 */
pub fn receive_effect_node_resume_event(
    mut pending: ResMut<EffectNodePendingEvents>,
    mut event_reader: EventReader<EffectNodeResumeEvent>,
) {
    for event in event_reader.read() {
        pending.pending_resume.push(event.node);
    }
}

pub fn effect_node_pause_event<T: EffectNode + Component>(
    mut query: Query<&mut EffectNodeTickState, With<T>>,
    mut event_reader: EventReader<EffectNodePauseEvent>,
) {
    for event in event_reader.read() {
        if let Ok(mut tick_state) = query.get_mut(event.node) {
            if *tick_state == EffectNodeTickState::Paused {
                continue;
            }
            info!(
                "node {} pause: {:?}",
                std::any::type_name::<T>(),
                event.node
            );
            *tick_state = EffectNodeTickState::Paused;
        }
    }
}

pub fn effect_node_resume_event<T: EffectNode + Component>(
    mut query: Query<&mut EffectNodeTickState, With<T>>,
    mut event_reader: EventReader<EffectNodeResumeEvent>,
) {
    for event in event_reader.read() {
        if let Ok(mut tick_state) = query.get_mut(event.node) {
            if *tick_state == EffectNodeTickState::Paused {
                info!(
                    "node {} resume: {:?}",
                    std::any::type_name::<T>(),
                    event.node
                );
                *tick_state = EffectNodeTickState::Ticked;
            }
        }
    }
}
