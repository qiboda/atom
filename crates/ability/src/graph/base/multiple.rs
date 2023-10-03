use std::any::TypeId;

use bevy::{prelude::*, reflect::Reflect};
use once_cell::sync::OnceCell;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::{EffectGraphContext, EffectPinKey},
    event::{
        effect_node_pause_event, effect_node_resume_event, node_can_pause, node_can_resume,
        node_can_start, EffectNodePendingEvents, EffectNodeStartEvent,
    },
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodeExecuteState, EffectNodePin,
        EffectNodePinGroup, EffectNodeTickState, EffectNodeUuid,
    },
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeMultiplePlugin {}

impl Plugin for EffectNodeMultiplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                effect_node_start_event.run_if(node_can_start()),
                effect_node_pause_event::<EffectNodeMultiple>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeMultiple>.run_if(node_can_resume()),
            ),
        );
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
        static CELL: OnceCell<Vec<EffectNodeExecGroup>> = OnceCell::new();
        CELL.get_or_init(|| {
            vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeMultiple::INPUT_EXEC_START,
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
            }]
        })
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        static CELL: OnceCell<Vec<EffectNodeExecGroup>> = OnceCell::new();
        CELL.get_or_init(|| {
            vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeMultiple::OUTPUT_EXEC_FINISH,
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeMultiple::OUTPUT_PIN_C,
                    pin_type: TypeId::of::<f32>(),
                }],
            }]
        })
    }
}

impl EffectNode for EffectNodeMultiple {}

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
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

fn effect_node_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeMultiple>>,
    mut graph_query: Query<&mut EffectGraphContext>,
    mut events: EventWriter<EffectNodeStartEvent>,
    pending: Res<EffectNodePendingEvents>,
) {
    for node_entity in pending.pending_start.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeMultiple>(),
                node_entity
            );

            let mut graph_context = graph_query.get_mut(parent.get()).unwrap();

            let a_input_key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeMultiple::INPUT_PIN_A,
            };
            let a_value = graph_context.get_input_value(&a_input_key);

            let b_input_key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeMultiple::INPUT_PIN_B,
            };
            let b_value = graph_context.get_input_value(&b_input_key);

            let mut c = EffectValue::F32(0.0);
            if let (Some(&EffectValue::F32(a)), Some(&EffectValue::F32(b))) = (a_value, b_value) {
                c = EffectValue::F32(a * b);
            }

            let c_output_key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeMultiple::OUTPUT_PIN_C,
            };

            if let Some(c_value) = graph_context.get_input_value_mut(&c_output_key) {
                *c_value = c;
            }

            let key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeMultiple::OUTPUT_EXEC_FINISH,
            };
            if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
                for entity in entities.iter() {
                    if let EffectValue::Entity(entity) = entity {
                        events.send(EffectNodeStartEvent::new(*entity));
                    }
                }
            }
        }
    }
}
