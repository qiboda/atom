use bevy::prelude::{Entity, Event};

#[derive(Event)]
pub enum EffectEvent {
    Start(Entity),
    Abort(Entity),
    Pause,
    Resume,
}
