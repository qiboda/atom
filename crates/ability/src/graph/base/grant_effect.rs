use std::ops::{Deref, Not};

use bevy::prelude::*;

use crate::{
    bundle::{EffectBundleTrait, ReflectEffectBundleTrait},
    effect::timer::EffectTime,
    graph::{
        blackboard::EffectValue,
        bundle::EffectNodeBaseBundle,
        context::{EffectGraphContext, EffectPinKey},
        event::{
            effect_node_pause_event, effect_node_resume_event, node_can_pause, node_can_resume,
            node_can_start, EffectNodePendingEvents, EffectNodeStartEvent,
        },
        node::{EffectNode, EffectNodeExecuteState, EffectNodeTickState, EffectNodeUuid},
    },
    impl_effect_node_pin_group,
};

///////////////////////// Plugin /////////////////////////

/// 类似于添加buff等效果。
#[derive(Debug, Default)]
pub struct EffectNodeGrantEffectPlugin {}

impl Plugin for EffectNodeGrantEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                effect_node_start_event.run_if(node_can_start()),
                effect_node_pause_event::<EffectNodeGrantEffect>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeGrantEffect>.run_if(node_can_resume()),
            ),
        )
        .add_systems(PostUpdate, react_on_remove_effect);
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component, Reflect)]
pub struct EffectNodeGrantEffect {
    pub effects: Vec<Entity>,
}

impl_effect_node_pin_group!(EffectNodeGrantEffect,
    input => (
        start, pins => (effect_bundle: Box<dyn EffectBundleTrait>)
    )
    output => (
        start, pins => (start_effect_entity: Entity),
        finish, pins => (end_effect_entity: Entity)
    )
);

impl EffectNode for EffectNodeGrantEffect {}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug, Default)]
pub struct EffectNodeGrantEffectBundle {
    pub node: EffectNodeGrantEffect,
    pub base: EffectNodeBaseBundle,
}

impl EffectNodeGrantEffectBundle {
    pub fn new() -> Self {
        Self {
            node: EffectNodeGrantEffect::default(),
            base: EffectNodeBaseBundle {
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

fn effect_node_start_event(
    mut commands: Commands,
    mut query: Query<(
        &mut EffectNodeGrantEffect,
        &EffectNodeUuid,
        &mut EffectNodeExecuteState,
        &Parent,
    )>,
    mut graph_query: Query<&mut EffectGraphContext>,
    pending: Res<EffectNodePendingEvents>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for node_entity in pending.pending_start.iter() {
        if let Ok((mut node, node_uuid, mut state, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeGrantEffect>(),
                node_entity
            );

            let mut graph_context = graph_query.get_mut(parent.get()).unwrap();

            let effect_value = graph_context.get_input_value(&EffectPinKey::new(
                *node_entity,
                *node_uuid,
                EffectNodeGrantEffect::INPUT_PIN_EFFECT_BUNDLE,
            ));

            if let Some(EffectValue::BoxReflect(v)) = effect_value {
                let read_guard = type_registry.read();
                let reflect_a_trait = read_guard
                    .get_type_data::<ReflectEffectBundleTrait>(v.type_id())
                    .unwrap();

                let effect_bundle: &dyn EffectBundleTrait = reflect_a_trait.get(v.deref()).unwrap();
                let effect_entity = effect_bundle.spawn_bundle(&mut commands);
                node.effects.push(effect_entity);

                // execute next node
                let entity_key = EffectPinKey {
                    node: *node_entity,
                    node_uuid: *node_uuid,
                    key: EffectNodeGrantEffect::OUTPUT_PIN_START_EFFECT_ENTITY,
                };

                graph_context.insert_output_value(entity_key, EffectValue::Entity(effect_entity));
                graph_context.exec_next_nodes(
                    *node_entity,
                    *node_uuid,
                    EffectNodeGrantEffect::OUTPUT_EXEC_START,
                    &mut event_writer,
                );

                if node.effects.is_empty().not() {
                    *state = EffectNodeExecuteState::Actived;
                }
            }
        }
    }
}

fn react_on_remove_effect(
    mut graph_query: Query<&mut EffectGraphContext>,
    mut query: Query<(
        Entity,
        &mut EffectNodeGrantEffect,
        &EffectNodeUuid,
        &mut EffectNodeExecuteState,
        &Parent,
    )>,
    mut removed_effects: RemovedComponents<EffectTime>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
) {
    for (node_entity, mut grant_effect, node_uuid, mut node_state, parent) in query.iter_mut() {
        if *node_state == EffectNodeExecuteState::Idle {
            continue;
        }

        // set idle state before remove effect(delay a frame to set idle),
        // avoid to graph be removed before next node active.
        if grant_effect.effects.is_empty() {
            *node_state = EffectNodeExecuteState::Idle;
        }

        let mut remove_success = vec![];
        grant_effect.effects.retain(|effect_entity| {
            for removed_effect_entity in removed_effects.read() {
                if *effect_entity == removed_effect_entity {
                    remove_success.push(removed_effect_entity);
                    return false;
                }
            }
            true
        });

        let mut graph_context = graph_query.get_mut(parent.get()).unwrap();
        for removed in remove_success.iter() {
            // execute next node
            let entity_key = EffectPinKey {
                node: node_entity,
                node_uuid: *node_uuid,
                key: EffectNodeGrantEffect::OUTPUT_PIN_END_EFFECT_ENTITY,
            };

            graph_context.insert_output_value(entity_key, EffectValue::Entity(*removed));

            graph_context.exec_next_nodes(
                node_entity,
                *node_uuid,
                EffectNodeGrantEffect::OUTPUT_EXEC_FINISH,
                &mut event_writer,
            );
        }
    }
}
