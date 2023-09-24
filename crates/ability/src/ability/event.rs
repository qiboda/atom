use bevy::{prelude::{Event, Entity}, reflect::Reflect};

#[derive(Event)]
pub enum AbilityEvent {
    Start(AbilityStartCommand),
    Abort(AbilityAbortCommand),
    Pause(AbilityPauseCommand),
    Resume(AbilityResumeCommand),
}

#[derive(Debug)]
pub struct AbilityStartCommand {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
    pub data: Option<Box<dyn Reflect>>,
}

#[derive(Debug)]
pub struct AbilityAbortCommand {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
}

#[derive(Debug)]
pub struct AbilityPauseCommand {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
}

#[derive(Debug)]
pub struct AbilityResumeCommand {
    pub owner: Entity,
    pub caster: Entity,
    pub ability: Entity,
}