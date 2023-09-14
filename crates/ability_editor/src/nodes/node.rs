use bevy::prelude::{default, Component};

pub trait EffectNode {
    fn start(&mut self);
    fn clear(&mut self);
    fn abort(&mut self);

    fn update(&mut self);

    fn pause(&mut self);

    fn resume(&mut self);
}

pub trait EffectStaticNode: EffectNode {}

pub trait EffectDynamicNode: EffectNode {}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub enum EffectNodeState {
    #[default]
    Idle,
    Running,
    Paused,
    Aborted,
    // when all children node is finished, the graph to set this idle.
    Finished,
}
