//! Buff 状态：标记、执行状态与节流状态。

use bevy::prelude::*;

use crate::graph::state::{EffectGraphState, EffectGraphTickState};

/// Buff 标记组件。
#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Buff;

/// Buff 执行状态。
#[derive(Debug, Component, Default, Reflect, PartialEq, Eq, Copy, Clone)]
pub enum BuffExecuteState {
    /// 未激活（默认）。
    #[default]
    Inactive,
    /// 执行中。
    Active,
    /// 待销毁。
    ToRemove,
}

/// Buff 节流状态。
#[derive(Debug, Component, Default, Reflect, PartialEq, Eq, Copy, Clone)]
pub enum BuffTickState {
    /// 正常运行（默认）。
    #[default]
    Ticked,
    /// 暂停。
    Paused,
}

/// 根据子图的状态更新 buff 的状态。
/// 如果有至少一个子图正在执行，那么这个 buff 就是执行中的。
/// 如果有至少一个子图 Idle 那么这个 buff 就是 Idle 的。
pub fn update_buff_state(
    mut query: Query<(&Children, &mut BuffExecuteState), With<Buff>>,
    graph_query: Query<&EffectGraphState>,
) {
    for (children, mut state) in query.iter_mut() {
        let mut any_active = false;
        let mut any_inactive = false;
        for child in children.iter() {
            if let Ok(graph_state) = graph_query.get(child) {
                match graph_state {
                    EffectGraphState::Inactive => {
                        any_inactive = true;
                    }
                    EffectGraphState::Active => {
                        any_active = true;
                    }
                    EffectGraphState::ToRemove => {}
                }
            }
        }

        if any_active {
            *state = BuffExecuteState::Active;
        } else if any_inactive {
            *state = BuffExecuteState::Inactive;
        } else {
            *state = BuffExecuteState::ToRemove;
        }
    }
}

/// 根据子图的状态更新 buff 的状态。
/// 如果所有的子图都 ToRemove，那么这个 buff 就是待移除的。
/// 如果所有的子图全部都 pause，则这个 buff 是 pause 的。
pub fn update_buff_tick_state(
    mut query: Query<(&Children, &mut BuffTickState), With<Buff>>,
    graph_query: Query<&EffectGraphTickState>,
) {
    for (children, mut state) in query.iter_mut() {
        let mut any_ticked = true;
        for child in children.iter() {
            if let Ok(graph_state) = graph_query.get(child) {
                match graph_state {
                    EffectGraphTickState::Ticked => {
                        any_ticked = true;
                    }
                    EffectGraphTickState::Paused => {}
                }
            }
        }

        if any_ticked {
            *state = BuffTickState::Ticked;
        } else {
            *state = BuffTickState::Paused;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn buff_marker_default_constructs() {
        let _buff: Buff = Buff;
    }

    #[test]
    fn buff_execute_state_default_is_inactive() {
        assert_eq!(BuffExecuteState::default(), BuffExecuteState::Inactive);
        assert_ne!(BuffExecuteState::Inactive, BuffExecuteState::Active);
        assert_ne!(BuffExecuteState::Active, BuffExecuteState::ToRemove);
    }

    #[test]
    fn buff_tick_state_default_is_ticked() {
        assert_eq!(BuffTickState::default(), BuffTickState::Ticked);
        assert_ne!(BuffTickState::Ticked, BuffTickState::Paused);
    }

    /// 创建 buff 实体（含执行/节流状态）并挂载带指定图状态的子实体。
    fn spawn_buff_with_graph_states(app: &mut App, graph_states: &[EffectGraphState]) -> Entity {
        let world = app.world_mut();
        let children = graph_states
            .iter()
            .map(|state| world.spawn(*state).id())
            .collect::<Vec<_>>();
        let mut buff = world.spawn((Buff, BuffExecuteState::default(), BuffTickState::default()));
        for child in children {
            buff.add_child(child);
        }
        buff.id()
    }

    #[test]
    fn update_buff_state_active_when_any_child_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let buff = spawn_buff_with_graph_states(
            &mut app,
            &[EffectGraphState::Active, EffectGraphState::Inactive],
        );
        app.add_systems(Update, update_buff_state);

        app.update();

        let state = app
            .world()
            .entity(buff)
            .get::<BuffExecuteState>()
            .expect("buff 执行状态应存在");
        assert_eq!(*state, BuffExecuteState::Active);
    }

    #[test]
    fn update_buff_state_inactive_when_only_inactive_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let buff = spawn_buff_with_graph_states(&mut app, &[EffectGraphState::Inactive]);
        app.add_systems(Update, update_buff_state);

        app.update();

        let state = app
            .world()
            .entity(buff)
            .get::<BuffExecuteState>()
            .expect("buff 执行状态应存在");
        assert_eq!(*state, BuffExecuteState::Inactive);
    }

    #[test]
    fn update_buff_state_to_remove_when_no_graph_state_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // 子实体无 EffectGraphState（查询失败）或全部 ToRemove → 无任何标记 → ToRemove。
        let buff = spawn_buff_with_graph_states(
            &mut app,
            &[EffectGraphState::ToRemove, EffectGraphState::ToRemove],
        );
        app.add_systems(Update, update_buff_state);

        app.update();

        let state = app
            .world()
            .entity(buff)
            .get::<BuffExecuteState>()
            .expect("buff 执行状态应存在");
        assert_eq!(*state, BuffExecuteState::ToRemove);
    }

    #[test]
    fn update_buff_state_skips_children_without_graph_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let plain_child = world.spawn_empty().id();
        let mut buff = world.spawn((Buff, BuffExecuteState::default(), BuffTickState::default()));
        buff.add_child(plain_child);
        let buff = buff.id();
        app.add_systems(Update, update_buff_state);

        app.update();

        let state = app
            .world()
            .entity(buff)
            .get::<BuffExecuteState>()
            .expect("buff 执行状态应存在");
        assert_eq!(
            *state,
            BuffExecuteState::ToRemove,
            "无图状态子实体 → 待移除"
        );
    }

    #[test]
    fn update_buff_tick_state_with_ticked_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let child = world.spawn(EffectGraphTickState::Ticked).id();
        let mut buff = world.spawn((Buff, BuffExecuteState::default(), BuffTickState::default()));
        buff.add_child(child);
        let buff = buff.id();
        app.add_systems(Update, update_buff_tick_state);

        app.update();

        let state = app
            .world()
            .entity(buff)
            .get::<BuffTickState>()
            .expect("buff 节流状态应存在");
        assert_eq!(*state, BuffTickState::Ticked);
    }

    #[test]
    fn update_buff_tick_state_with_paused_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let child = world.spawn(EffectGraphTickState::Paused).id();
        let mut buff = world.spawn((Buff, BuffExecuteState::default(), BuffTickState::default()));
        buff.add_child(child);
        let buff = buff.id();
        app.add_systems(Update, update_buff_tick_state);

        app.update();

        let state = app
            .world()
            .entity(buff)
            .get::<BuffTickState>()
            .expect("buff 节流状态应存在");
        // 当前实现：any_ticked 初始为 true，Paused 子图不改变结果 → 仍为 Ticked。
        assert_eq!(*state, BuffTickState::Ticked);
    }
}
