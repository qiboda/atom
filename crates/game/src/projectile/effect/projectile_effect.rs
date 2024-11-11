// TODO 等待BSN实现完毕后再进行
// use bevy::{prelude::*, time::Time};

// use ability::{
//     graph::{
//         context::{EffectGraphContext, InstantEffectNodeMap},
//         event::EffectNodeExecEvent,
//         executor::EffectGraphExecutor,
//         node::{
//             pin::EffectNodeExec, EffectNode, EffectNodeExecuteState, EffectNodeId, StateEffectNode,
//         },
//         pin::EffectNodeExecPin,
//         state::EffectGraphTickState,
//         EffectGraphUpdateSystemSet,
//     },
//     impl_effect_node_pin_group,
// };

// #[derive(Debug)]
// pub struct EffectNodeProjectilePlugin;

// impl Plugin for EffectNodeProjectilePlugin {
//     fn build(&self, app: &mut App) {
//         app.register_type::<EffectNodeProjectile>()
//             .observe(trigger_effect_node_event)
//             .add_systems(
//                 Update,
//                 update_projectile.in_set(EffectGraphUpdateSystemSet::UpdateNode),
//             );
//     }
// }

// #[derive(Clone, Debug, Default, Reflect)]
// pub struct EffectNodeProjectileState {
//     pub entity: f32,
// }

// // TODO 这个类可以泛型化。
// #[derive(Clone, Debug, Default, Component, Reflect)]
// #[reflect(Component)]
// pub struct EffectNodeProjectile {
//     pub states: Vec<EffectNodeProjectileState>,
// }

// impl EffectNode for EffectNodeProjectile {}

// impl StateEffectNode for EffectNodeProjectile {}

// impl_effect_node_pin_group!(EffectNodeProjectile,
//     input => (
//         start => (projectile_bundle: impl Bundle)
//     )
//     output => (
//         start => (),
//         hit => (count: usize),
//         finish => ()
//     )
// );

// fn trigger_effect_node_event(
//     trigger: Trigger<EffectNodeExecEvent>,
//     mut query: Query<(
//         &mut EffectNodeProjectile,
//         &mut EffectNodeExecuteState,
//         &Parent,
//     )>,
//     mut graph_query: Query<(&EffectGraphContext, &mut EffectGraphExecutor)>,
//     instant_nodes: Res<InstantEffectNodeMap>,
// ) {
//     let pin = trigger.event().input_exec_pin;
//     let EffectNodeId::Entity(entity) = pin.node_id else {
//         return;
//     };

//     info!("trigger_node_event: entry {:?}", pin);

//     if let Ok((mut node, mut state, parent)) = query.get_mut(entity) {
//         if let Ok((context, mut executor)) = graph_query.get_mut(parent.get()) {
//             if let EffectNodeProjectile::INPUT_EXEC_START = pin.exec.name {
//                 let duration_value = context.get_input_value_type_from_node::<&f32>(
//                     entity,
//                     &*node,
//                     EffectNodeProjectile::INPUT_SLOT_DURATION,
//                 );

//                 if let Some(duration) = duration_value {
//                     node.states
//                         .push(EffectNodeProjectileState { elapse: *duration });
//                 }

//                 if *state == EffectNodeExecuteState::Idle {
//                     *state = EffectNodeExecuteState::Actived;
//                 }

//                 executor.start_push_output_pin(
//                     EffectNodeExecPin {
//                         node_id: entity.into(),
//                         exec: EffectNodeProjectile::OUTPUT_EXEC_START.into(),
//                     },
//                     context,
//                     &instant_nodes,
//                 );
//             }
//         }
//     }
// }

// fn update_projectile(
//     mut graph_query: Query<(
//         &EffectGraphContext,
//         &mut EffectGraphExecutor,
//         &EffectGraphTickState,
//     )>,
//     mut query: Query<(
//         Entity,
//         &mut EffectNodeProjectile,
//         &mut EffectNodeExecuteState,
//         &Parent,
//     )>,
//     instant_map: Res<InstantEffectNodeMap>,
//     time: Res<Time>,
// ) {
//     for (entity, mut node, mut node_state, parent) in query.iter_mut() {
//         if *node_state == EffectNodeExecuteState::Idle {
//             continue;
//         }

//         let (context, mut executor, tick_state) = graph_query.get_mut(parent.get()).unwrap();
//         if *tick_state != EffectGraphTickState::Ticked {
//             continue;
//         }

//         for state in node.states.iter_mut() {
//             state.elapse -= time.delta_seconds();
//             if state.elapse <= 0.0 {
//                 executor.start_push_output_pin(
//                     EffectNodeExecPin {
//                         node_id: EffectNodeId::Entity(entity),
//                         exec: EffectNodeExec {
//                             name: EffectNodeProjectile::OUTPUT_EXEC_FINISH,
//                         },
//                     },
//                     context,
//                     &instant_map,
//                 );
//             }
//         }

//         node.states.retain(|state| state.elapse > 0.0);

//         if node.states.is_empty() {
//             *node_state = EffectNodeExecuteState::Idle;
//         }
//     }
// }
