use std::any::TypeId;

use bevy::prelude::*;

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
pub struct EffectNodeLogPlugin {}

impl Plugin for EffectNodeLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                effect_node_start_event.run_if(node_can_start()),
                effect_node_pause_event::<EffectNodeLog>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeLog>.run_if(node_can_resume()),
            ),
        );
    }
}

impl EffectNodeLogPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeLog {}

impl EffectNodeLog {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_MESSAGE: &'static str = "message";

    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
    pub const OUTPUT_PIN_MESSAGE: &'static str = "message";
}

impl EffectNodePinGroup for EffectNodeLog {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        static CELL: OnceCell<Vec<EffectNodeExecGroup>> = OnceCell::new();
        CELL.get_or_init(|| {
            vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeLog::INPUT_EXEC_START,
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeLog::INPUT_PIN_MESSAGE,
                    pin_type: TypeId::of::<String>(),
                }],
            }]
        })
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        static CELL: OnceCell<Vec<EffectNodeExecGroup>> = OnceCell::new();
        CELL.get_or_init(|| {
            vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeLog::OUTPUT_EXEC_FINISH,
                },
                pins: vec![],
            }]
        })
    }
}

impl EffectNode for EffectNodeLog {}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug, Default)]
pub struct LogNodeBundle {
    pub effect_node: EffectNodeLog,
    pub effect_node_base: EffectNodeBaseBundle,
}

impl LogNodeBundle {
    pub fn new() -> Self {
        Self {
            effect_node: EffectNodeLog::default(),
            effect_node_base: EffectNodeBaseBundle {
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

fn effect_node_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeLog>>,
    graph_query: Query<&EffectGraphContext>,
    mut events: ResMut<Events<EffectNodeStartEvent>>,
    pending: Res<EffectNodePendingEvents>,
) {
    for node_entity in pending.pending_start.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeLog>(),
                node_entity
            );

            let graph_context = graph_query.get(parent.get()).unwrap();

            let duration_input_key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeLog::INPUT_PIN_MESSAGE,
            };
            let duration_value = graph_context.get_input_value(&duration_input_key);

            if let Some(EffectValue::String(message)) = duration_value {
                info!("{}", message);
            }

            let key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeLog::OUTPUT_EXEC_FINISH,
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
