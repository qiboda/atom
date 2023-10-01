use std::any::TypeId;

use lazy_static::lazy_static;

use bevy::{
    prelude::{App, Bundle, Commands, Component, Entity, Plugin, EventWriter},
    reflect::Reflect,
};

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::{EffectGraphContext, EffectPinKey},
    event::{EffectEvent, EffectNodeEventPlugin},
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
        EffectNodeState, EffectNodeUuid,
    },
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeMultiplePlugin {}

impl Plugin for EffectNodeMultiplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectNodeEventPlugin::<EffectNodeMultiple>::default())
            // .add_systems(Update, update_msg);
            ;
    }
}

///////////////////////// Node Event /////////////////////////

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeMultipleEndExec {
    pub entities: Vec<Entity>,
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component, Reflect)]
pub struct EffectNodeMultiple {}

impl EffectNodeMultiple {
    const INPUT_EXEC_START: &'static str = "start";
    const INPUT_PIN_A: &'static str = "a";
    const INPUT_PIN_B: &'static str = "b";

    const OUTPUT_EXEC_FINISH: &'static str = "finish";
    const OUTPUT_PIN_C: &'static str = "c";
}

impl EffectNodePinGroup for EffectNodeMultiple {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeMultiple::INPUT_EXEC_START
                },
                pins: vec![
                    EffectNodePin {
                        name: EffectNodeMultiple::INPUT_PIN_A,
                        pin_type: TypeId::of::<f32>(),
                    },
                    EffectNodePin {
                        name: EffectNodeMultiple::INPUT_PIN_B,
                        pin_type: TypeId::of::<f32>(),
                    },
                ],
            }];
        }
        &INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeMultiple::OUTPUT_EXEC_FINISH
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeMultiple::OUTPUT_PIN_C,
                    pin_type: TypeId::of::<f32>(),
                }],
            }];
        }

        &OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeMultiple {
    fn start(
        &mut self,
        _commands: &mut Commands,
        node_entity: Entity,
        node_uuid: &EffectNodeUuid,
        _node_state: &mut EffectNodeState,
        graph_context: &mut EffectGraphContext,
        event_writer: &mut EventWriter<EffectEvent>,
    ) {
        let a_input_key = EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeMultiple::INPUT_PIN_A,
        };
        let a_value = graph_context.get_input_value(&a_input_key);

        let b_input_key = EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeMultiple::INPUT_PIN_B,
        };
        let b_value = graph_context.get_input_value(&b_input_key);

        let mut c = EffectValue::F32(0.0);
        if let (Some(&EffectValue::F32(a)), Some(&EffectValue::F32(b))) = (a_value, b_value) {
            c = EffectValue::F32(a * b);
        }

        let c_output_key = EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeMultiple::OUTPUT_PIN_C,
        };

        if let Some(c_value) = graph_context.get_input_value_mut(&c_output_key) {
            *c_value = c;
        }

        if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeMultiple::OUTPUT_EXEC_FINISH,
        }) {
            for entity in entities.iter() {
                if let EffectValue::Entity(entity) = entity {
                    event_writer.send(EffectEvent::Start(*entity));
                }
            }
        }
    }

    fn clear(&mut self) {}

    fn abort(&mut self) {}

    fn update(&mut self) {}

    fn pause(&mut self) {}

    fn resume(&mut self) {}
}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug, Default)]
pub struct MultipleNodeBundle {
    effect_node: EffectNodeMultiple,
    effect_node_base: EffectNodeBaseBundle,
}

impl MultipleNodeBundle {
    pub fn new() -> Self {
        Self {
            effect_node: EffectNodeMultiple::default(),
            effect_node_base: EffectNodeBaseBundle {
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}
