//! Effect 事件：开始/中断/暂停/恢复。

use bevy::{
    prelude::{Entity, Event, EventReader, Query},
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
pub fn receive_start_effect(
    mut event_reader: EventReader<EffectStartEvent>,
    mut effect_query: Query<&mut EffectState>,
) {
    for event in event_reader.read() {
        if let Ok(mut state) = effect_query.get_mut(event.effect) {
            if *state == EffectState::Inactive {
                *state = EffectState::CheckCanActive;
            }
        }
    }
}
