//! Effect 事件与 observer：效果开始/中断/暂停/恢复。

use bevy::{
    prelude::{Entity, Event, On, Query},
    reflect::Reflect,
};

use super::state::EffectState;

/// Effect 开始事件：携带效果实体与可选反射数据。
#[derive(Debug, Event)]
pub struct EffectStartEvent {
    /// 效果实体。
    pub effect: Entity,
    /// 可选的开始数据。
    pub data: Option<Box<dyn Reflect>>,
}

/// Effect 中断事件。
#[derive(Debug, Event)]
pub struct EffectAbortEvent {
    /// 所有者实体。
    pub owner: Entity,
    /// 施法者实体。
    pub caster: Entity,
    /// 技能实体。
    pub ability: Entity,
}

/// Effect 暂停事件。
#[derive(Debug, Event)]
pub struct EffectPauseEvent {
    /// 所有者实体。
    pub owner: Entity,
    /// 施法者实体。
    pub caster: Entity,
    /// 技能实体。
    pub ability: Entity,
}

/// Effect 恢复事件。
#[derive(Debug, Event)]
pub struct EffectResumeEvent {
    /// 所有者实体。
    pub owner: Entity,
    /// 施法者实体。
    pub caster: Entity,
    /// 技能实体。
    pub ability: Entity,
}

/// 处理 [`EffectStartEvent`]：非激活的效果进入 `CheckCanActive` 检查阶段。
pub fn trigger_effect_start(
    trigger: On<EffectStartEvent>,
    mut effect_query: Query<&mut EffectState>,
) {
    let effect_entity = trigger.event().effect;
    if let Ok(mut state) = effect_query.get_mut(effect_entity) {
        if *state == EffectState::Inactive {
            *state = EffectState::CheckCanActive;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn effect_start_event_moves_inactive_to_check_can_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(trigger_effect_start);

        let entity = app.world_mut().spawn(EffectState::Inactive).id();
        app.add_systems(Update, move |mut commands: Commands| {
            commands.trigger(EffectStartEvent {
                effect: entity,
                data: None,
            });
        });
        app.update();

        let state = app
            .world()
            .entity(entity)
            .get::<EffectState>()
            .expect("effect state must exist");
        assert_eq!(*state, EffectState::CheckCanActive);
    }
}
