//! Effect 状态层标签：效果各阶段所需/禁用/增删的标签与相关系统。

use std::ops::Not;

use atom_layertag::{
    container_op::{
        LayerTagContainerConditionRequired, LayerTagContainerConditionWithout,
        LayerTagContainerOpAdd, LayerTagContainerOpRemove,
    },
    count_container::CountLayerTagContainer,
};
use bevy::prelude::*;

use crate::stateset::StateLayerTagContainer;

use super::state::EffectState;

/// Effect 开始所需的状态层标签容器（全部满足才可开始）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectStartRequiredLayerTagContainer(pub CountLayerTagContainer);

/// Effect 开始禁用的状态层标签容器（存在任一即不可开始）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectStartDisableLayerTagContainer(pub CountLayerTagContainer);

/// Effect 中断所需的状态层标签容器（全部满足才可中断）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectAbortRequiredLayerTagContainer(pub CountLayerTagContainer);

/// Effect 中断禁用的状态层标签容器（存在任一即不可中断）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectAbortDisableLayerTagContainer(pub CountLayerTagContainer);

/// Effect 结束后是否回滚状态层标签。
#[derive(Debug, Default, Reflect, PartialEq, Eq, Clone)]
pub enum EffectLayerTagContainerRevert {
    /// 不回滚（默认）。
    #[default]
    No,
    /// 回滚。
    Yes,
}

impl From<bool> for EffectLayerTagContainerRevert {
    fn from(value: bool) -> Self {
        if value {
            EffectLayerTagContainerRevert::Yes
        } else {
            EffectLayerTagContainerRevert::No
        }
    }
}

/// Effect 开始时要添加的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectAddedLayerTagContainer {
    /// 要添加的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: EffectLayerTagContainerRevert,
}

/// Effect 开始时要移除的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectRemovedLayerTagContainer {
    /// 要移除的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: EffectLayerTagContainerRevert,
}

/// 检查待激活效果的开始条件：不满足所需/禁用标签则置回非激活。
pub fn effect_tag_start_check_system(
    state_set_query: Query<&StateLayerTagContainer>,
    mut query: Query<(
        &ChildOf,
        &mut EffectState,
        &EffectStartRequiredLayerTagContainer,
        &EffectStartDisableLayerTagContainer,
    )>,
) {
    for (parent, mut effect_state, required_tag, disable_tag) in query.iter_mut() {
        if *effect_state == EffectState::CheckCanActive {
            let state_layer_tag_container = state_set_query
                .get(parent.parent())
                .expect("state layer tag container must exist on parent");

            let can_start = state_layer_tag_container
                .0
                .condition(LayerTagContainerConditionRequired, &required_tag.0)
                && state_layer_tag_container
                    .0
                    .condition(LayerTagContainerConditionWithout, &disable_tag.0);
            if can_start.not() {
                *effect_state = EffectState::Inactive;
            }
        }
    }
}

/// 对刚激活的效果应用状态层标签增删。
pub fn effect_tag_start_apply_system(
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    query: Query<(
        &ChildOf,
        &EffectState,
        &EffectAddedLayerTagContainer,
        &EffectRemovedLayerTagContainer,
    )>,
) {
    for (parent, effect_state, added_tag, removed_tag) in query.iter() {
        if *effect_state == EffectState::ActiveBefore {
            let mut state_layer_tag_container = state_set_query
                .get_mut(parent.parent())
                .expect("state layer tag container must exist on parent");

            state_layer_tag_container
                .0
                .receive_op(LayerTagContainerOpAdd, &added_tag.layer_tag_container);

            state_layer_tag_container
                .0
                .receive_op(LayerTagContainerOpRemove, &removed_tag.layer_tag_container);
        }
    }
}

/// 对即将失活的效果回滚其标记为可回滚的标签增删。
pub fn effect_tag_revert_apply_system(
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    query: Query<(
        &ChildOf,
        &EffectState,
        &EffectAddedLayerTagContainer,
        &EffectRemovedLayerTagContainer,
    )>,
) {
    for (parent, effect_state, added_tag, removed_tag) in query.iter() {
        if *effect_state == EffectState::BeforeInactive {
            let mut state_layer_tag_container = state_set_query
                .get_mut(parent.parent())
                .expect("state layer tag container must exist on parent");

            if added_tag.revert == EffectLayerTagContainerRevert::Yes {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpRemove, &added_tag.layer_tag_container);
            }

            if removed_tag.revert == EffectLayerTagContainerRevert::Yes {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpAdd, &removed_tag.layer_tag_container);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stateset::StateLayerTagRegistry;
    use atom_layertag::container_op::LayerTagContainer;
    use bevy::MinimalPlugins;

    /// 构造含单个已注册标签的计数容器。
    fn tag_container(raw: &str) -> CountLayerTagContainer {
        let mut registry = StateLayerTagRegistry::default();
        registry.0.register_raw(raw);
        let layertag = registry
            .0
            .request_from_raw(raw)
            .expect("已注册标签必须可取");
        let mut container = CountLayerTagContainer::default();
        container.add_layertag(layertag);
        container
    }

    /// 构造 effect 实体：父实体持有状态层容器，effect 子实体持有状态与标签组件。
    fn spawn_effect(
        app: &mut App,
        parent_tags: CountLayerTagContainer,
        effect_state: EffectState,
        required: CountLayerTagContainer,
        disable: CountLayerTagContainer,
        added: CountLayerTagContainer,
        removed: CountLayerTagContainer,
        added_revert: EffectLayerTagContainerRevert,
        removed_revert: EffectLayerTagContainerRevert,
    ) -> Entity {
        let world = app.world_mut();
        let parent = world.spawn(StateLayerTagContainer(parent_tags)).id();
        world
            .spawn((
                effect_state,
                EffectStartRequiredLayerTagContainer(required),
                EffectStartDisableLayerTagContainer(disable),
                EffectAddedLayerTagContainer {
                    layer_tag_container: added,
                    revert: added_revert,
                },
                EffectRemovedLayerTagContainer {
                    layer_tag_container: removed,
                    revert: removed_revert,
                },
            ))
            .set_parent_in_place(parent)
            .id()
    }

    #[test]
    fn effect_layer_tag_container_revert_from_bool() {
        assert_eq!(
            EffectLayerTagContainerRevert::default(),
            EffectLayerTagContainerRevert::No
        );
        assert_eq!(
            EffectLayerTagContainerRevert::from(true),
            EffectLayerTagContainerRevert::Yes
        );
        assert_eq!(
            EffectLayerTagContainerRevert::from(false),
            EffectLayerTagContainerRevert::No
        );
    }

    #[test]
    fn tag_container_structs_construct_with_default() {
        let _start_required = EffectStartRequiredLayerTagContainer::default();
        let _start_disable = EffectStartDisableLayerTagContainer::default();
        let _abort_required = EffectAbortRequiredLayerTagContainer::default();
        let _abort_disable = EffectAbortDisableLayerTagContainer::default();
        let added = EffectAddedLayerTagContainer::default();
        let removed = EffectRemovedLayerTagContainer::default();
        assert_eq!(added.revert, EffectLayerTagContainerRevert::No);
        assert_eq!(removed.revert, EffectLayerTagContainerRevert::No);
        assert!(added.layer_tag_container.iter_layertag().next().is_none());
        assert!(removed.layer_tag_container.iter_layertag().next().is_none());
    }

    #[test]
    fn start_check_marks_inactive_when_required_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let parent_tags = tag_container("state.a");
        // 所需标签 "state.required" 未在父容器中出现 → 不可开始。
        let effect = spawn_effect(
            &mut app,
            parent_tags,
            EffectState::CheckCanActive,
            tag_container("state.required"),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_start_check_system);

        app.update();

        let state = app
            .world()
            .entity(effect)
            .get::<EffectState>()
            .expect("effect 状态应存在");
        assert_eq!(*state, EffectState::Inactive);
    }

    #[test]
    fn start_check_marks_inactive_when_disable_present() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // 父容器含禁用标签 "state.b" → 不可开始。
        let effect = spawn_effect(
            &mut app,
            tag_container("state.b"),
            EffectState::CheckCanActive,
            CountLayerTagContainer::default(),
            tag_container("state.b"),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_start_check_system);

        app.update();

        let state = app
            .world()
            .entity(effect)
            .get::<EffectState>()
            .expect("effect 状态应存在");
        assert_eq!(*state, EffectState::Inactive);
    }

    #[test]
    fn start_check_keeps_state_when_conditions_met() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            tag_container("state.a"),
            EffectState::CheckCanActive,
            tag_container("state.a"),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_start_check_system);

        app.update();

        let state = app
            .world()
            .entity(effect)
            .get::<EffectState>()
            .expect("effect 状态应存在");
        assert_eq!(
            *state,
            EffectState::CheckCanActive,
            "条件满足时保持检查阶段"
        );
    }

    #[test]
    fn start_check_skips_non_check_can_active_states() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            tag_container("state.a"),
            EffectState::Active,
            tag_container("state.missing"),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_start_check_system);

        app.update();

        let state = app
            .world()
            .entity(effect)
            .get::<EffectState>()
            .expect("effect 状态应存在");
        assert_eq!(*state, EffectState::Active, "非检查状态不得被修改");
    }

    #[test]
    #[should_panic(expected = "state layer tag container must exist on parent")]
    fn start_check_panics_when_parent_container_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let world = app.world_mut();
        // 父实体存在但不带 StateLayerTagContainer → expect 触发 panic。
        let parent = world.spawn_empty().id();
        let _effect = world
            .spawn((
                EffectState::CheckCanActive,
                EffectStartRequiredLayerTagContainer::default(),
                EffectStartDisableLayerTagContainer::default(),
            ))
            .set_parent_in_place(parent)
            .id();
        app.add_systems(Update, effect_tag_start_check_system);

        app.update();
    }

    #[test]
    fn start_apply_adds_and_removes_tags_on_active_before() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            tag_container("state.existing"),
            EffectState::ActiveBefore,
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            tag_container("state.added"),
            tag_container("state.existing"),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_start_apply_system);

        app.update();

        let parent = app
            .world()
            .entity(effect)
            .get::<ChildOf>()
            .expect("effect 应有父实体")
            .parent();
        let container = app
            .world()
            .entity(parent)
            .get::<StateLayerTagContainer>()
            .expect("父实体应有状态层容器");
        assert!(
            container
                .0
                .iter_layertag()
                .any(|t| t.raw_layertag() == "state.added"),
            "added 标签应被添加到父容器"
        );
        assert!(
            !container
                .0
                .iter_layertag()
                .any(|t| t.raw_layertag() == "state.existing"),
            "removed 标签应从父容器移除"
        );
    }

    #[test]
    fn start_apply_skips_non_active_before_states() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            CountLayerTagContainer::default(),
            EffectState::Active,
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            tag_container("state.added"),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_start_apply_system);

        app.update();

        let parent = app
            .world()
            .entity(effect)
            .get::<ChildOf>()
            .expect("effect 应有父实体")
            .parent();
        let container = app
            .world()
            .entity(parent)
            .get::<StateLayerTagContainer>()
            .expect("父实体应有状态层容器");
        assert!(
            container.0.iter_layertag().next().is_none(),
            "非 ActiveBefore 不得应用标签"
        );
    }

    #[test]
    fn revert_apply_removes_added_and_restores_removed_on_before_inactive() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            tag_container("state.kept"),
            EffectState::BeforeInactive,
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            tag_container("state.added"),
            tag_container("state.restore"),
            EffectLayerTagContainerRevert::Yes,
            EffectLayerTagContainerRevert::Yes,
        );
        app.add_systems(Update, effect_tag_revert_apply_system);

        app.update();

        let parent = app
            .world()
            .entity(effect)
            .get::<ChildOf>()
            .expect("effect 应有父实体")
            .parent();
        let container = app
            .world()
            .entity(parent)
            .get::<StateLayerTagContainer>()
            .expect("父实体应有状态层容器");
        let tags = container
            .0
            .iter_layertag()
            .map(|t| t.raw_layertag())
            .collect::<Vec<_>>();
        assert!(tags.contains(&"state.kept".to_string()), "无关标签不受影响");
        assert!(
            !tags.contains(&"state.added".to_string()),
            "revert=Yes 的 added 标签应被移除"
        );
        assert!(
            tags.contains(&"state.restore".to_string()),
            "revert=Yes 的 removed 标签应被恢复"
        );
    }

    #[test]
    fn revert_apply_skips_non_revertable_tags() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            tag_container("state.added"),
            EffectState::BeforeInactive,
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            tag_container("state.added"),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::No,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_revert_apply_system);

        app.update();

        let parent = app
            .world()
            .entity(effect)
            .get::<ChildOf>()
            .expect("effect 应有父实体")
            .parent();
        let container = app
            .world()
            .entity(parent)
            .get::<StateLayerTagContainer>()
            .expect("父实体应有状态层容器");
        let tags = container
            .0
            .iter_layertag()
            .map(|t| t.raw_layertag())
            .collect::<Vec<_>>();
        assert!(
            tags.contains(&"state.added".to_string()),
            "revert=No 时不得回滚"
        );
    }

    #[test]
    fn revert_apply_skips_non_before_inactive_states() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let effect = spawn_effect(
            &mut app,
            tag_container("state.added"),
            EffectState::Active,
            CountLayerTagContainer::default(),
            CountLayerTagContainer::default(),
            tag_container("state.added"),
            CountLayerTagContainer::default(),
            EffectLayerTagContainerRevert::Yes,
            EffectLayerTagContainerRevert::No,
        );
        app.add_systems(Update, effect_tag_revert_apply_system);

        app.update();

        let parent = app
            .world()
            .entity(effect)
            .get::<ChildOf>()
            .expect("effect 应有父实体")
            .parent();
        let container = app
            .world()
            .entity(parent)
            .get::<StateLayerTagContainer>()
            .expect("父实体应有状态层容器");
        let tags = container
            .0
            .iter_layertag()
            .map(|t| t.raw_layertag())
            .collect::<Vec<_>>();
        assert!(
            tags.contains(&"state.added".to_string()),
            "非 BeforeInactive 不得回滚"
        );
    }
}
