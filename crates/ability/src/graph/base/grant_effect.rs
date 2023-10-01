use bevy::{
    prelude::{
        App, Bundle, Commands, Component, Entity, EventWriter, Parent, Plugin, PostUpdate, Query, RemovedComponents,
    },
    reflect::Reflect,
};

use lazy_static::lazy_static;

use crate::{
    bundle::AbilityBundle,
    effect::{
        timer::{
            AbilityEffectDelay, AbilityEffectEnd, AbilityEffectLoop, AbilityEffectStart,
            AbilityEffectTimer,
        },
        AbilityEffect,
    },
    graph::{
        blackboard::{BlackBoardValue, EffectValue},
        bundle::EffectNodeBaseBundle,
        context::{EffectGraphContext, EffectPinKey},
        event::{EffectEvent, EffectNodeEventPlugin},
        node::{
            EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
            EffectNodeState, EffectNodeUuid,
        },
    },
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeGrantEffectPlugin {}

impl Plugin for EffectNodeGrantEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectNodeEventPlugin::<EffectNodeGrantEffect>::default())
            .add_systems(PostUpdate, react_on_remove_effect);
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeGrantEffect {
    pub effects: Vec<Entity>,
}

impl EffectNodeGrantEffect {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_EFFECT_BUNDLE: &'static str = "effect_bundle";
    pub const INPUT_PIN_EFFECT_TIMER_VEC: &'static str = "effect_timer_vec";

    pub const OUTPUT_EXEC_START: &'static str = "start";
    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
    pub const OUTPUT_PIN_STARTR_EFFECT_ENTITY: &'static str = "start_effect_entity";
    pub const OUTPUT_PIN_FINISH_EFFECT_ENTITY: &'static str = "end_effect_entity";
}

impl EffectNodePinGroup for EffectNodeGrantEffect {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeGrantEffect::INPUT_EXEC_START
                },
                pins: vec![
                    EffectNodePin {
                        name: EffectNodeGrantEffect::INPUT_PIN_EFFECT_BUNDLE,
                        pin_type: std::any::TypeId::of::<AbilityEffect>(),
                    },
                    EffectNodePin {
                        name: EffectNodeGrantEffect::INPUT_PIN_EFFECT_TIMER_VEC,
                        pin_type: std::any::TypeId::of::<Vec<Box<dyn AbilityEffectTimer>>>(),
                    },
                ],
            }];
        };
        &INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![
                EffectNodeExecGroup {
                    exec: EffectNodeExec {
                        name: EffectNodeGrantEffect::OUTPUT_EXEC_START
                    },
                    pins: vec![EffectNodePin {
                        name: EffectNodeGrantEffect::OUTPUT_PIN_STARTR_EFFECT_ENTITY,
                        pin_type: std::any::TypeId::of::<Vec<Box<dyn AbilityEffectTimer>>>(),
                    },],
                },
                EffectNodeExecGroup {
                    exec: EffectNodeExec {
                        name: EffectNodeGrantEffect::OUTPUT_EXEC_FINISH
                    },
                    pins: vec![EffectNodePin {
                        name: EffectNodeGrantEffect::OUTPUT_PIN_FINISH_EFFECT_ENTITY,
                        pin_type: std::any::TypeId::of::<Vec<Box<dyn AbilityEffectTimer>>>(),
                    },],
                }
            ];
        }
        &OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeGrantEffect {
    fn start(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        uuid: &EffectNodeUuid,
        _node_state: &mut EffectNodeState,
        graph_context: &mut EffectGraphContext,
        event_writer: &mut EventWriter<EffectEvent>,
    ) {
        let effect_input_key = EffectPinKey {
            node: entity,
            node_id: *uuid,
            key: EffectNodeGrantEffect::INPUT_PIN_EFFECT_BUNDLE,
        };
        let effect_value = graph_context.get_input_value(&effect_input_key);
        let effect_timer_input_key = EffectPinKey {
            node: entity,
            node_id: *uuid,
            key: EffectNodeGrantEffect::INPUT_PIN_EFFECT_TIMER_VEC,
        };
        let effect_timer_value = graph_context.get_input_value(&effect_timer_input_key);

        if let (Some(effect), Some(effect_timer_vec)) = (effect_value, effect_timer_value) {
            if let Ok(effect) = effect.get::<&Box<dyn Reflect>>() {
                let effect_bundle = effect.downcast_ref::<AbilityBundle>().unwrap();
                let mut entity_command = commands.spawn(effect_bundle.clone());
                if let Ok(timer_vec) = effect_timer_vec.get::<&Vec<EffectValue>>() {
                    for timer in timer_vec.iter() {
                        if let Ok(timer_value) = timer.get::<&Box<dyn Reflect>>() {
                            if let Some(timer) = timer_value.downcast_ref::<AbilityEffectLoop>() {
                                entity_command.insert(timer.clone());
                            } else if let Some(timer) =
                                timer_value.downcast_ref::<AbilityEffectStart>()
                            {
                                entity_command.insert(timer.clone());
                            } else if let Some(timer) =
                                timer_value.downcast_ref::<AbilityEffectEnd>()
                            {
                                entity_command.insert(timer.clone());
                            } else if let Some(timer) =
                                timer_value.downcast_ref::<AbilityEffectDelay>()
                            {
                                entity_command.insert(timer.clone());
                            }
                        }
                    }
                }
                let effect_entity = entity_command.id();
                self.effects.push(effect_entity);

                // execute next node
                let entity_key = EffectPinKey {
                    node: entity,
                    node_id: *uuid,
                    key: EffectNodeGrantEffect::OUTPUT_PIN_STARTR_EFFECT_ENTITY,
                };

                graph_context.insert_output_value(entity_key, EffectValue::Entity(effect_entity));

                let key = EffectPinKey {
                    node: entity,
                    node_id: *uuid,
                    key: EffectNodeGrantEffect::OUTPUT_EXEC_START,
                };
                if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
                    for entity in entities.iter() {
                        if let EffectValue::Entity(entity) = entity {
                            event_writer.send(EffectEvent::Start(*entity));
                        }
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        assert!(self.effects.is_empty());
        self.effects.clear();
    }

    fn abort(&mut self) {}

    fn update(&mut self) {}

    fn pause(&mut self) {}

    fn resume(&mut self) {}
}

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
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

fn react_on_remove_effect(
    mut graph_query: Query<&mut EffectGraphContext>,
    mut query: Query<(
        Entity,
        &mut EffectNodeGrantEffect,
        &EffectNodeUuid,
        &EffectNodeState,
        &Parent,
    )>,
    mut removed_effects: RemovedComponents<AbilityEffect>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (node_entity, mut grant_effect, node_uuid, _state, parent) in query.iter_mut() {
        let mut remove_success = vec![];
        grant_effect.effects.retain(|effect_entity| {
            for removed_effect_entity in removed_effects.iter() {
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
                node_id: *node_uuid,
                key: EffectNodeGrantEffect::OUTPUT_PIN_STARTR_EFFECT_ENTITY,
            };

            graph_context.insert_output_value(entity_key, EffectValue::Entity(*removed));

            let key = EffectPinKey {
                node: node_entity,
                node_id: *node_uuid,
                key: EffectNodeGrantEffect::OUTPUT_EXEC_FINISH,
            };
            if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
                for entity in entities.iter() {
                    if let EffectValue::Entity(entity) = entity {
                        event_writer.send(EffectEvent::Start(*entity));
                    }
                }
            }
        }
    }
}
