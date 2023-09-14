use std::any::TypeId;

use bevy::{
    prelude::{
        default, App, Bundle, Component, Entity, EventWriter, Parent, Plugin, PreUpdate, Query,
        Update,
    },
    reflect::Reflect,
};

use crate::nodes::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    event::EffectEvent,
    graph::{EffectGraphContext, EffectOutputKey},
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
        EffectNodeState, EffectNodeUuid,
    },
    receive_effect_event,
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug)]
pub struct EffectNodeMultiplePlugin {}

impl Plugin for EffectNodeMultiplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, receive_effect_event::<EffectNodeMultiple>)
            .add_systems(Update, update_msg);
    }
}

impl EffectNodeMultiplePlugin {
    pub fn new() -> Self {
        Self {}
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
    const INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
        exec: EffectNodeExec { name: "start" },
        pins: vec![
            EffectNodePin {
                name: "a",
                pin_type: TypeId::of::<f32>(),
            },
            EffectNodePin {
                name: "b",
                pin_type: TypeId::of::<f32>(),
            },
        ],
    }];
    const OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
        exec: EffectNodeExec { name: "finish" },
        pins: vec![EffectNodePin {
            name: "c",
            pin_type: TypeId::of::<f32>(),
        }],
    }];
}

impl EffectNodePinGroup for EffectNodeMultiple {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        &Self::INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        &Self::OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeMultiple {
    fn start(&mut self) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn abort(&mut self) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn pause(&mut self) {
        todo!()
    }

    fn resume(&mut self) {
        todo!()
    }
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
            effect_node: EffectNodeMultiple { ..default() },
            effect_node_base: EffectNodeBaseBundle {
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

fn update_msg(
    mut graph_context: Query<&EffectGraphContext>,
    mut query: Query<(&EffectNodeMultiple, &mut EffectNodeState, &Parent)>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (node, mut state, parent) in query.iter_mut() {
        if *state == EffectNodeState::Running {
            let c = node.a + node.b;
            if let context = graph_context.get_mut(parent).unwrap() {
                context.outputs.insert(
                    EffectOutputKey {
                        node: parent,
                        output_key: "c".to_string(),
                    },
                    EffectValue::Float(c),
                );
            }

            *state = EffectNodeState::Finished;
            for entity in node.end_exec.entities.iter() {
                event_writer.send(EffectEvent::Start(*entity));
            }
        }
    }
}
