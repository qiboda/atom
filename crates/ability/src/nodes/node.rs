use bevy::prelude::Component;

pub trait EffectNode {
    fn start(&mut self);
    fn clear(&mut self);
}

pub trait EffectStaticNode: EffectNode {}

pub trait EffectDynamicNode: EffectNode {
    fn abort(&mut self);

    fn update(&mut self);

    fn pause(&mut self);

    fn resume(&mut self);
}

#[derive(Debug, Component, Copy, Clone, PartialEq, Eq)]
pub enum EffectNodeState {
    Idle,
    Running,
    Paused,
    Aborted,
    Finished,
}
