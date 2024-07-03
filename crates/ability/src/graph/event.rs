use std::sync::Arc;

use atom_utils::clone_entities::{CloneEntityTreeCommand, EntityTreeNode};
use bevy::{ecs::entity::EntityHashMap, prelude::*, utils::HashMap};

use crate::graph::pin::EffectNodeSlotPin;

use super::{
    blackboard::EffectValue,
    context::{EffectGraphContext, GraphRef, InstantEffectNodeMap},
    executor::EffectGraphExecutor,
    graph_map::{EffectGraphBuilderMap, EffectGraphMap, GraphClass},
    node::pin::{EffectNodeExec, EffectNodeSlot},
    pin::EffectNodeExecPin,
    state::{EffectGraphState, EffectGraphTickState},
    EffectGraphOwner,
};

/// run a graph node
#[derive(Debug, Event)]
pub struct EffectNodeExecEvent {
    pub input_exec_pin: EffectNodeExecPin,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct CloneEffectGraphStartEvent {
    pub graph_ref: GraphRef,
    pub destination_entity: Entity,
}

impl Default for CloneEffectGraphStartEvent {
    fn default() -> Self {
        Self {
            graph_ref: GraphRef::new(Entity::PLACEHOLDER),
            destination_entity: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Debug, Event)]
pub struct CloneEffectGraphEndEvent {
    pub destination_root_entity: Entity,
    pub old_new_entities: EntityHashMap<Entity>,
}

// 添加一个EffectGraph
#[derive(Debug, Event, Clone)]
pub struct EffectGraphAddEvent {
    pub graph_class: GraphClass,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct EffectGraphRemoveEvent;

#[derive(Debug, Event, Clone)]
pub struct EffectGraphExecEvent {
    pub entry_exec_pin: EffectNodeExec,
    pub execute_in_graph_state: Option<EffectGraphState>,
    pub slot_value_map: Option<HashMap<EffectNodeSlot, EffectValue>>,
}

#[derive(Debug, Event, Clone, Copy)]
pub struct EffectGraphTickableEvent {
    pub tickable: bool,
}

pub fn trigger_effect_graph_tickable(
    triger: Trigger<EffectGraphTickableEvent>,
    graph_owner_query: Query<&Children, With<EffectGraphOwner>>,
    mut graph_query: Query<&mut EffectGraphTickState>,
) {
    let graph_owner_entity = triger.entity();
    let Ok(children) = graph_owner_query.get(graph_owner_entity) else {
        return;
    };

    let event = triger.event();
    for child in children {
        if let Ok(mut state) = graph_query.get_mut(*child) {
            if event.tickable {
                *state = EffectGraphTickState::Ticked;
            } else {
                *state = EffectGraphTickState::Paused;
            }
            info!(
                "trigger effect graph tickable: {:?} => {:?} : {:?}",
                graph_owner_entity, child, state
            );
        }
    }
}

pub fn trigger_effect_graph_to_remove(
    triger: Trigger<EffectGraphRemoveEvent>,
    graph_owner_query: Query<&Children, With<EffectGraphOwner>>,
    mut graph_query: Query<&mut EffectGraphState>,
) {
    let graph_owner_entity = triger.entity();
    let Ok(children) = graph_owner_query.get(graph_owner_entity) else {
        return;
    };

    for child in children {
        if let Ok(mut state) = graph_query.get_mut(*child) {
            info!(
                "trigger effect graph remove: {:?} => {:?} ",
                graph_owner_entity, child
            );
            *state = EffectGraphState::ToRemove;
        }
    }
}

// 应该添加Ready状态，技能首先进入Ready状态(检查是否可以Start)，
// 在Ready Pin后执行Start ability or buff Event,或者不满足条件，进入Idle状态。
pub fn trigger_effect_graph_exec(
    trigger: Trigger<EffectGraphExecEvent>,
    mut commands: Commands,
    graph_owner_query: Query<&Children, With<EffectGraphOwner>>,
    mut graph_query: Query<(
        &mut EffectGraphContext,
        &mut EffectGraphExecutor,
        &EffectGraphState,
    )>,
    instant_map: Res<InstantEffectNodeMap>,
) {
    let graph_owner_entity = trigger.entity();
    let Ok(children) = graph_owner_query.get(graph_owner_entity) else {
        return;
    };

    info!(
        "trigger effect graph exec: {:?} => {:?} ",
        graph_owner_entity,
        trigger.event()
    );

    // NOTE: Ready exec pin must use "ready" name.
    // TODO: 写重复了。。。。。只需要节点复制就足够了，实例复制没必要。。。。。
    //      同样的，Effect Grahp state的设置也没有意义。
    // 以上有问题，如果技能需要还原，结束，中断等，多次技能使用同一个EffectGraph instance，
    // 会有问题。因为每个节点存储了多个状态，但是不知道应该还原，结束，中断哪些状态。
    if trigger.event().entry_exec_pin == "ready".into() {
        let mut clone_event = CloneEffectGraphStartEvent::default();
        let mut to_clone_graph = false;
        for child in children {
            if let Ok((context, mut _executor, state)) = graph_query.get(*child) {
                match state {
                    EffectGraphState::Inactive => {
                        to_clone_graph = false;
                        break;
                    }
                    EffectGraphState::Active => {
                        clone_event.graph_ref = context.get_graph_ref().unwrap();
                        to_clone_graph = true;
                    }
                    EffectGraphState::ToRemove => {}
                }
            }
        }

        if to_clone_graph {
            let new_graph_instance = commands.spawn_empty().set_parent(graph_owner_entity).id();
            clone_event.destination_entity = new_graph_instance;
            commands.trigger_targets(clone_event, new_graph_instance);
            info!(
                "trigger effect graph exec to clone graph: {:?} => {:?} => {:?} ",
                graph_owner_entity,
                trigger.event(),
                clone_event
            );

            commands.trigger_targets(trigger.event().clone(), graph_owner_entity);
            return;
        }
    }

    for child in children {
        if let Ok((mut context, mut executor, state)) = graph_query.get_mut(*child) {
            let mut execute_start = || {
                info!(
                    "trigger effect graph exec: {:?} => {:?} in state {:?} ",
                    graph_owner_entity,
                    trigger.event(),
                    state
                );

                let entry_entity = context.get_entry_node().unwrap();
                if let Some(slot_value_map) = trigger.event().slot_value_map.as_ref() {
                    for (slot, value) in slot_value_map {
                        context.insert_output_value(
                            EffectNodeSlotPin {
                                node_id: entry_entity.into(),
                                slot: *slot,
                            },
                            value.clone().into(),
                        );
                    }
                }
                executor.start_push_output_pin(
                    EffectNodeExecPin {
                        node_id: context.get_entry_node().unwrap().into(),
                        exec: trigger.event().entry_exec_pin,
                    },
                    &context,
                    &instant_map,
                );
            };

            match trigger.event().execute_in_graph_state {
                Some(execute_state) => {
                    if *state == execute_state {
                        execute_start();
                    }
                }
                None => {
                    execute_start();
                }
            }
        }
    }
}

pub fn trigger_effect_graph_add(
    trigger: Trigger<EffectGraphAddEvent>,
    mut commands: Commands,
    mut graph_map: ResMut<EffectGraphMap>,
    graph_builder_map: Res<EffectGraphBuilderMap>,
    mut instant_map: ResMut<InstantEffectNodeMap>,
) {
    let graph_owner_entity = trigger.entity();
    let event = trigger.event();

    let mut graph_ref = graph_map.get_graph(event.graph_class.clone());
    if graph_ref.is_none() {
        let graph_builder = graph_builder_map.get_effect_graph_builder(&event.graph_class);
        match graph_builder {
            Some(builder) => {
                let graph = builder.build(&mut commands, &mut instant_map);
                graph_ref = Some(GraphRef::new(graph));
                info!("build graph template: {:?}", graph);
                graph_map.insert_graph(event.graph_class.clone(), graph_ref.unwrap());
            }
            None => {
                error!("graph builder not found: {}", event.graph_class);
            }
        }
    }

    let Some(graph_ref) = graph_ref else {
        return;
    };

    let new_graph_entity = commands.spawn_empty().set_parent(graph_owner_entity).id();
    commands.trigger_targets(
        CloneEffectGraphStartEvent {
            graph_ref,
            destination_entity: new_graph_entity,
        },
        new_graph_entity,
    );
}

pub fn trigger_clone_effect_graph_start(
    trigger: Trigger<CloneEffectGraphStartEvent>,
    mut commands: Commands,
    query_children: Query<&Children>,
) {
    let event = trigger.event();
    assert_ne!(trigger.entity(), Entity::PLACEHOLDER);

    let graph_ref = event.graph_ref;
    let new_graph_entity = event.destination_entity;

    // clone all entity and component to effect graph.
    let entity_tree_node = EntityTreeNode::from_entity_recursive(
        &mut commands,
        graph_ref.get_entity(),
        Some(new_graph_entity),
        &query_children,
    );
    let mut old_new_entites = entity_tree_node.recursive_get_entities_map();
    old_new_entites.remove(&graph_ref.get_entity());

    let clone_entities = CloneEntityTreeCommand(Arc::new(entity_tree_node));
    commands.add(clone_entities);
    commands.trigger_targets(
        CloneEffectGraphEndEvent {
            destination_root_entity: new_graph_entity,
            old_new_entities: old_new_entites,
        },
        new_graph_entity,
    );

    info!("clone_effect_graph_start");
}

/// add to effect entity observer
/// 替换所有graph context中的entities，替换为新的entities
pub fn trigger_clone_effect_graph_end(
    trigger: Trigger<CloneEffectGraphEndEvent>,
    mut commands: Commands,
    mut query: Query<&mut EffectGraphContext>,
) {
    let event = trigger.event();
    assert_ne!(trigger.entity(), Entity::PLACEHOLDER);

    commands
        .entity(trigger.entity())
        .insert(EffectGraphExecutor::default());

    info!("clone entities: {:?}", event.old_new_entities);

    let mut context = query.get_mut(trigger.entity()).unwrap();
    // info!("context: before");
    // dbg!(&context);
    context.replace_state_entities(event.old_new_entities.clone());
    // info!("context: after");
    // dbg!(&context);

    info!("clone_effect_graph_end");
}
