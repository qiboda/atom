use bevy::{
    ecs::message::Message,
    prelude::{Entity, Event},
};

#[derive(Event, Message)]
pub enum EffectEvent {
    Start(Entity),
    Abort(Entity),
    Pause,
    Resume,
}
