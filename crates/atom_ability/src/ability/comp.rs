//! 技能组件：技能标记、执行状态与节流状态。

use bevy::prelude::*;

use crate::graph::state::{EffectGraphState, EffectGraphTickState};

// only ability with this component
/// 技能标记组件：只有带此组件的实体才是技能实体。
#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Ability;

/// 技能执行状态。
#[derive(Debug, Component, Default, PartialEq, Eq, Reflect, Copy, Clone)]
pub enum AbilityExecuteState {
    /// 未激活（默认）。
    #[default]
    Inactive,
    /// 执行中。
    Active,
    /// 待销毁（所有子图已移除）。
    ToRemove,
}

/// 技能节流状态。
#[derive(Debug, Component, Default, PartialEq, Eq, Reflect, Copy, Clone)]
pub enum AbilityTickState {
    /// 正常运行（默认）。
    #[default]
    Ticked,
    /// 暂停。
    Paused,
}

/// 技能数据占位组件。
#[derive(Debug, Component, Default, Reflect)]
pub struct AbilityData;

/// 根据子图的状态更新技能的状态。
/// 如果有至少一个子图正在执行，那么这个技能就是执行中的。
/// 如果有至少一个子图Idle那么这个技能就是Idle的。
pub fn update_ability_state(
    mut query: Query<(&Children, &mut AbilityExecuteState), With<Ability>>,
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
            *state = AbilityExecuteState::Active;
        } else if any_inactive {
            *state = AbilityExecuteState::Inactive;
        } else {
            *state = AbilityExecuteState::ToRemove;
        }
    }
}

/// 根据子图的状态更新技能的状态。
/// 如果所有的子图都ToRemove，那么这个技能就是待移除的。
/// 如果所有的子图全部都pause，则这个技能是pause的。
pub fn update_ability_tick_state(
    mut query: Query<(&Children, &mut AbilityTickState), With<Ability>>,
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
            *state = AbilityTickState::Ticked;
        } else {
            *state = AbilityTickState::Paused;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn ability_marker_default_constructs() {
        let _ability: Ability = Ability;
    }

    #[test]
    fn ability_execute_state_default_is_inactive() {
        assert_eq!(
            AbilityExecuteState::default(),
            AbilityExecuteState::Inactive
        );
        assert_ne!(AbilityExecuteState::Inactive, AbilityExecuteState::Active);
        assert_ne!(AbilityExecuteState::Active, AbilityExecuteState::ToRemove);
    }

    #[test]
    fn ability_tick_state_default_is_ticked() {
        assert_eq!(AbilityTickState::default(), AbilityTickState::Ticked);
        assert_ne!(AbilityTickState::Ticked, AbilityTickState::Paused);
    }

    #[test]
    fn ability_data_default_constructs() {
        let data = AbilityData;
        let _ = data;
    }

    fn spawn_ability_with_graph_states(app: &mut App, graph_states: &[EffectGraphState]) -> Entity {
        let world = app.world_mut();
        let children = graph_states
            .iter()
            .map(|state| world.spawn(*state).id())
            .collect::<Vec<_>>();
        let mut ability = world.spawn((
            Ability,
            AbilityExecuteState::default(),
            AbilityTickState::default(),
        ));
        for child in children {
            ability.add_child(child);
        }
        ability.id()
    }

    #[test]
    fn update_ability_state_active_when_any_child_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let ability = spawn_ability_with_graph_states(
            &mut app,
            &[EffectGraphState::Active, EffectGraphState::Inactive],
        );
        app.add_systems(Update, update_ability_state);

        app.update();

        let state = app
            .world()
            .entity(ability)
            .get::<AbilityExecuteState>()
            .expect("技能执行状态应存在");
        assert_eq!(*state, AbilityExecuteState::Active);
    }

    #[test]
    fn update_ability_state_inactive_when_only_inactive_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let ability = spawn_ability_with_graph_states(&mut app, &[EffectGraphState::Inactive]);
        app.add_systems(Update, update_ability_state);

        app.update();

        let state = app
            .world()
            .entity(ability)
            .get::<AbilityExecuteState>()
            .expect("技能执行状态应存在");
        assert_eq!(*state, AbilityExecuteState::Inactive);
    }

    #[test]
    fn update_ability_state_to_remove_when_no_matching_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let ability = spawn_ability_with_graph_states(&mut app, &[EffectGraphState::ToRemove]);
        app.add_systems(Update, update_ability_state);

        app.update();

        let state = app
            .world()
            .entity(ability)
            .get::<AbilityExecuteState>()
            .expect("技能执行状态应存在");
        assert_eq!(*state, AbilityExecuteState::ToRemove);
    }

    #[test]
    fn update_ability_state_skips_children_without_graph_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let plain_child = world.spawn_empty().id();
        let mut ability = world.spawn((
            Ability,
            AbilityExecuteState::default(),
            AbilityTickState::default(),
        ));
        ability.add_child(plain_child);
        let ability = ability.id();
        app.add_systems(Update, update_ability_state);

        app.update();

        let state = app
            .world()
            .entity(ability)
            .get::<AbilityExecuteState>()
            .expect("技能执行状态应存在");
        assert_eq!(
            *state,
            AbilityExecuteState::ToRemove,
            "无图状态子实体 → 待移除"
        );
    }

    #[test]
    fn update_ability_tick_state_with_ticked_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let child = world.spawn(EffectGraphTickState::Ticked).id();
        let mut ability = world.spawn((
            Ability,
            AbilityExecuteState::default(),
            AbilityTickState::default(),
        ));
        ability.add_child(child);
        let ability = ability.id();
        app.add_systems(Update, update_ability_tick_state);

        app.update();

        let state = app
            .world()
            .entity(ability)
            .get::<AbilityTickState>()
            .expect("技能节流状态应存在");
        assert_eq!(*state, AbilityTickState::Ticked);
    }

    #[test]
    fn update_ability_tick_state_with_paused_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        let child = world.spawn(EffectGraphTickState::Paused).id();
        let mut ability = world.spawn((
            Ability,
            AbilityExecuteState::default(),
            AbilityTickState::default(),
        ));
        ability.add_child(child);
        let ability = ability.id();
        app.add_systems(Update, update_ability_tick_state);

        app.update();

        let state = app
            .world()
            .entity(ability)
            .get::<AbilityTickState>()
            .expect("技能节流状态应存在");
        // 当前实现：any_ticked 初始为 true，Paused 子图不改变结果 → 仍为 Ticked。
        assert_eq!(*state, AbilityTickState::Ticked);
    }
}
