use std::ops::Not;

use bevy::{
    prelude::{Commands, Component, Entity, EventWriter, Query, Res},
    reflect::{reflect_trait, Reflect},
    time::Time,
};

use crate::graph::{
    context::{EffectGraphContext, GraphRef},
    event::EffectEvent,
};

use super::AbilityEffect;

#[reflect_trait]
pub trait AbilityEffectTimer {}

#[derive(Component, Debug, Reflect, Clone)]
pub struct AbilityEffectLoop {
    pub loop_duration: f32,
    pub graph: GraphRef,
}

impl AbilityEffectTimer for AbilityEffectLoop {}

#[derive(Component, Debug, Reflect, Clone)]
pub struct AbilityEffectDelay {
    pub delay: f32,
    pub graph: GraphRef,
}

impl AbilityEffectTimer for AbilityEffectDelay {}

#[derive(Component, Debug, Reflect, Clone)]
pub struct AbilityEffectStart {
    pub actived: bool,
    pub graph: GraphRef,
}

impl AbilityEffectTimer for AbilityEffectStart {}

#[derive(Component, Debug, Reflect, Clone)]
pub struct AbilityEffectEnd {
    pub actived: bool,
    pub graph: GraphRef,
}

impl AbilityEffectTimer for AbilityEffectEnd {}

pub fn update_ability_effect_timer_system(mut query: Query<&mut AbilityEffect>, time: Res<Time>) {
    for mut effect in query.iter_mut() {
        effect.elapse += time.delta_seconds();
    }
}

pub fn destroy_ability_effect(mut commands: Commands, query: Query<(Entity, &AbilityEffect)>) {
    for (entity, effect) in query.iter() {
        if effect.elapse >= effect.duration {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_ability_effect_start_timer(
    mut query: Query<(&AbilityEffect, Option<&mut AbilityEffectStart>)>,
    graph_query: Query<&EffectGraphContext>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (effect, start) in query.iter_mut() {
        if let Some(mut start) = start {
            if start.actived.not() && effect.elapse > 0.0 {
                let graph_context = graph_query.get(start.graph.get_entity()).unwrap();
                if let Some(entry_node) = graph_context.entry_node {
                    start.actived = true;
                    let event = EffectEvent::Start(entry_node);
                    event_writer.send(event);
                }
            }
        }
    }
}

pub fn update_ability_effect_end_timer(
    mut query: Query<(&AbilityEffect, Option<&mut AbilityEffectEnd>)>,
    graph_query: Query<&EffectGraphContext>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (effect, end) in query.iter_mut() {
        if let Some(mut end) = end {
            if end.actived.not() && effect.elapse >= effect.duration {
                let graph_context = graph_query.get(end.graph.get_entity()).unwrap();
                if let Some(entry_node) = graph_context.entry_node {
                    end.actived = true;
                    let event = EffectEvent::Start(entry_node);
                    event_writer.send(event);
                }
            }
        }
    }
}

pub fn update_ability_effect_loop_timer(
    mut query: Query<(&AbilityEffect, &mut AbilityEffectLoop)>,
    graph_query: Query<&EffectGraphContext>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (effect, mut loop_timer) in query.iter_mut() {
        if effect.elapse >= loop_timer.loop_duration {
            let graph_context = graph_query.get(loop_timer.graph.get_entity()).unwrap();
            if let Some(entry_node) = graph_context.entry_node {
                loop_timer.loop_duration += loop_timer.loop_duration;
                let event = EffectEvent::Start(entry_node);
                event_writer.send(event);
            }
        }
    }
}

pub fn update_ability_effect_delay_timer(
    mut query: Query<(&AbilityEffect, &mut AbilityEffectDelay)>,
    graph_query: Query<&EffectGraphContext>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (effect, mut delay_timer) in query.iter_mut() {
        if effect.elapse >= delay_timer.delay {
            let graph_context = graph_query.get(delay_timer.graph.get_entity()).unwrap();
            if let Some(entry_node) = graph_context.entry_node {
                delay_timer.delay += delay_timer.delay;
                let event = EffectEvent::Start(entry_node);
                event_writer.send(event);
            }
        }
    }
}
