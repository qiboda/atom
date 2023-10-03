use bevy::{
    prelude::{Entity, Event, EventReader, Query},
    reflect::Reflect,
};

use super::state::EffectState;

#[derive(Debug, Event)]
pub struct EffectStartEvent {
    pub effect: Entity,
    pub data: Option<Box<dyn Reflect>>,
}

#[derive(Debug, Event)]
pub struct EffectAbortEvent {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
}

#[derive(Debug, Event)]
pub struct EffectPauseEvent {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
}

#[derive(Debug, Event)]
pub struct EffectResumeEvent {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
}

pub fn receive_start_effect(
    mut event_reader: EventReader<EffectStartEvent>,
    mut effect_query: Query<&mut EffectState>,
) {
    for event in event_reader.iter() {
        if let Ok(mut state) = effect_query.get_mut(event.effect) {
            if *state == EffectState::Unactived {
                *state = EffectState::CheckCanActive;
            }
        }
    }
}
