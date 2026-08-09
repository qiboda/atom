//! Effect Graph 状态管理：图级执行状态与销毁清理。

use bevy::log::info;
use bevy::prelude::*;

use super::{context::EffectGraphContext, node::EffectNodeExecuteState};

/// Effect Graph 的节流状态：是否暂停执行。
#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum EffectGraphTickState {
    /// 正常运行（默认）。
    #[default]
    Ticked,
    /// 暂停执行。
    Paused,
}

/// Effect Graph 的生命周期状态。
#[derive(Debug, Component, Default, PartialEq, Eq, Hash, Reflect, Clone, Copy)]
#[reflect(Component)]
pub enum EffectGraphState {
    /// 未激活（默认）。
    #[default]
    Inactive,
    /// 激活执行中。
    Active,
    /// 待销毁：等待所有节点空闲后移除图实体。
    ToRemove,
}

/// 重置图状态：当所有状态节点处于 `Idle` 时，将 `Active` 的图置回 `Inactive`。
pub fn reset_effect_graph_state(
    mut query: Query<(&mut EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (mut state, context) in query.iter_mut() {
        match *state {
            EffectGraphState::Inactive => {}
            EffectGraphState::Active => {
                if context.state_nodes.iter().all(|node| {
                    if let Ok(node_state) = node_state_query.get(*node)
                        && *node_state == EffectNodeExecuteState::Idle
                    {
                        return true;
                    }
                    false
                }) {
                    *state = EffectGraphState::Inactive;
                }
            }
            EffectGraphState::ToRemove => {}
        }
    }
}

/// 销毁待移除的图：对 `ToRemove` 且所有状态节点均空闲的图，despawn 其实体。
pub fn update_to_despawn_effect_graph(
    mut commands: Commands,
    query: Query<(Entity, &EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (graph_entity, state, context) in query.iter() {
        if *state == EffectGraphState::ToRemove
            && context.state_nodes.iter().all(|node| {
                if let Ok(node_state) = node_state_query.get(*node)
                    && *node_state == EffectNodeExecuteState::Idle
                {
                    return true;
                }
                false
            })
        {
            commands.entity(graph_entity).despawn();
            info!("despawn graph: {:?}", graph_entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::EffectNodeExecuteState;
    use bevy::MinimalPlugins;

    #[test]
    fn effect_graph_state_default_is_inactive() {
        assert_eq!(EffectGraphState::default(), EffectGraphState::Inactive);
        assert_ne!(EffectGraphState::Inactive, EffectGraphState::Active);
        assert_ne!(EffectGraphState::Active, EffectGraphState::ToRemove);
    }

    #[test]
    fn effect_graph_tick_state_default_is_ticked() {
        assert_eq!(
            EffectGraphTickState::default(),
            EffectGraphTickState::Ticked
        );
        assert_ne!(EffectGraphTickState::Ticked, EffectGraphTickState::Paused);
    }

    fn spawn_graph_with_state_nodes(app: &mut App, graph_state: EffectGraphState) -> Entity {
        let world = app.world_mut();
        let node_a = world.spawn(EffectNodeExecuteState::Idle).id();
        let node_b = world.spawn(EffectNodeExecuteState::Idle).id();
        let mut context = EffectGraphContext::new();
        context.insert_state_node(node_a);
        context.insert_state_node(node_b);
        world.spawn((graph_state, context)).id()
    }

    #[test]
    fn reset_effect_graph_state_returns_active_to_inactive_when_all_idle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let graph = spawn_graph_with_state_nodes(&mut app, EffectGraphState::Active);
        app.add_systems(Update, reset_effect_graph_state);

        app.update();

        assert_eq!(
            *app.world()
                .entity(graph)
                .get::<EffectGraphState>()
                .expect("图状态应存在"),
            EffectGraphState::Inactive
        );
    }

    #[test]
    fn reset_effect_graph_state_keeps_active_while_node_busy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let busy_node = world.spawn(EffectNodeExecuteState::Active).id();
        let idle_node = world.spawn(EffectNodeExecuteState::Idle).id();
        let mut context = EffectGraphContext::new();
        context.insert_state_node(busy_node);
        context.insert_state_node(idle_node);
        let graph = world.spawn((EffectGraphState::Active, context)).id();
        app.add_systems(Update, reset_effect_graph_state);

        app.update();

        assert_eq!(
            *app.world()
                .entity(graph)
                .get::<EffectGraphState>()
                .expect("图状态应存在"),
            EffectGraphState::Active,
            "存在执行中节点时图不得回到 Inactive"
        );
    }

    #[test]
    fn reset_effect_graph_state_leaves_inactive_and_to_remove_untouched() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let graph = spawn_graph_with_state_nodes(&mut app, EffectGraphState::Inactive);
        let to_remove = spawn_graph_with_state_nodes(&mut app, EffectGraphState::ToRemove);
        app.add_systems(Update, reset_effect_graph_state);

        app.update();

        assert_eq!(
            *app.world()
                .entity(graph)
                .get::<EffectGraphState>()
                .expect("图状态应存在"),
            EffectGraphState::Inactive
        );
        assert_eq!(
            *app.world()
                .entity(to_remove)
                .get::<EffectGraphState>()
                .expect("图状态应存在"),
            EffectGraphState::ToRemove
        );
    }

    #[test]
    fn update_to_despawn_effect_graph_despawns_idle_to_remove_graph() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let graph = spawn_graph_with_state_nodes(&mut app, EffectGraphState::ToRemove);
        app.add_systems(Update, update_to_despawn_effect_graph);

        app.update();

        assert!(
            app.world().get_entity(graph).is_err(),
            "全部状态节点空闲的 ToRemove 图应被 despawn"
        );
    }

    #[test]
    fn update_to_despawn_effect_graph_keeps_busy_to_remove_graph() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let busy_node = world.spawn(EffectNodeExecuteState::Active).id();
        let mut context = EffectGraphContext::new();
        context.insert_state_node(busy_node);
        let graph = world.spawn((EffectGraphState::ToRemove, context)).id();
        app.add_systems(Update, update_to_despawn_effect_graph);

        app.update();

        assert!(
            app.world().get_entity(graph).is_ok(),
            "存在执行中节点时图不得被 despawn"
        );
    }
}
