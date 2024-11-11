use bevy::{prelude::*, time::Time};

use crate::{
    graph::{
        context::{EffectGraphContext, InstantEffectNodeMap},
        event::EffectNodeExecEvent,
        executor::EffectGraphExecutor,
        node::{
            pin::EffectNodeExec, EffectNode, EffectNodeExecuteState, EffectNodeId, StateEffectNode,
        },
        pin::EffectNodeExecPin,
        state::EffectGraphTickState,
        EffectGraphUpdateSystemSet,
    },
    impl_effect_node_pin_group,
};

#[derive(Debug)]
pub struct EffectNodeTimerPlugin;

impl Plugin for EffectNodeTimerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectNodeTimer>()
            .observe(trigger_effect_node_event)
            .add_systems(
                Update,
                update_timer.in_set(EffectGraphUpdateSystemSet::UpdateNode),
            );
    }
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct EffectNodeTimerState {
    pub elapse: f32,
}

#[derive(Clone, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectNodeTimer {
    pub states: Vec<EffectNodeTimerState>,
}

impl EffectNode for EffectNodeTimer {}

impl StateEffectNode for EffectNodeTimer {}

impl_effect_node_pin_group!(EffectNodeTimer,
    input => (
        start => (duration: f32)
    )
    output => (
        start => (),
        finish => ()
    )
);

fn trigger_effect_node_event(
    trigger: Trigger<EffectNodeExecEvent>,
    mut query: Query<(&mut EffectNodeTimer, &mut EffectNodeExecuteState, &Parent)>,
    mut graph_query: Query<(&EffectGraphContext, &mut EffectGraphExecutor)>,
    instant_nodes: Res<InstantEffectNodeMap>,
) {
    let pin = trigger.event().input_exec_pin;
    let EffectNodeId::Entity(entity) = pin.node_id else {
        return;
    };

    if let Ok((mut node, mut state, parent)) = query.get_mut(entity) {
        info!("trigger_node_event: timer {:?}", pin);

        if let Ok((context, mut executor)) = graph_query.get_mut(parent.get()) {
            if let EffectNodeTimer::INPUT_EXEC_START = pin.exec.name {
                let duration_value = context.get_input_value_type_from_node::<&f32>(
                    entity,
                    &*node,
                    EffectNodeTimer::INPUT_SLOT_DURATION,
                );

                if let Some(duration) = duration_value {
                    node.states.push(EffectNodeTimerState { elapse: *duration });
                }

                if *state == EffectNodeExecuteState::Idle {
                    *state = EffectNodeExecuteState::Active;
                }

                executor.start_push_output_pin(
                    EffectNodeExecPin {
                        node_id: entity.into(),
                        exec: EffectNodeTimer::OUTPUT_EXEC_START.into(),
                    },
                    context,
                    &instant_nodes,
                );
            }
        }
    }
}

fn update_timer(
    mut graph_query: Query<(
        &EffectGraphContext,
        &mut EffectGraphExecutor,
        &EffectGraphTickState,
    )>,
    mut query: Query<(
        Entity,
        &mut EffectNodeTimer,
        &mut EffectNodeExecuteState,
        &Parent,
    )>,
    instant_map: Res<InstantEffectNodeMap>,
    time: Res<Time>,
) {
    for (entity, mut node, mut node_state, parent) in query.iter_mut() {
        if *node_state == EffectNodeExecuteState::Idle {
            continue;
        }

        let (context, mut executor, tick_state) = graph_query.get_mut(parent.get()).unwrap();
        if *tick_state != EffectGraphTickState::Ticked {
            continue;
        }

        for state in node.states.iter_mut() {
            state.elapse -= time.delta_seconds();
            if state.elapse <= 0.0 {
                executor.start_push_output_pin(
                    EffectNodeExecPin {
                        node_id: EffectNodeId::Entity(entity),
                        exec: EffectNodeExec {
                            name: EffectNodeTimer::OUTPUT_EXEC_FINISH,
                        },
                    },
                    context,
                    &instant_map,
                );
            }
        }

        node.states.retain(|state| state.elapse > 0.0);

        if node.states.is_empty() {
            *node_state = EffectNodeExecuteState::Idle;
        }
    }
}
