use std::any::TypeId;

use bevy::{prelude::Component, reflect::reflect_trait, utils::Uuid};

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

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub struct EffectNodeUuid {
    pub uuid: Uuid,
}

impl EffectNodeUuid {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

pub struct EffectNodeExecGroup {
    pub exec: EffectNodeExec,
    pub pins: Vec<EffectNodePin>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct EffectNodeExec {
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectNodePin {
    pub name: &'static str,
    pub pin_type: TypeId,
}

#[reflect_trait]
pub trait EffectNodePinGroup {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    fn get_input_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_input_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    fn get_input_pin_group_num(&self) -> usize {
        self.get_input_pin_group().len()
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup>;

    fn get_output_pin_group_by_name(&self, name: &str) -> Option<&EffectNodeExecGroup> {
        self.get_output_pin_group()
            .iter()
            .find(|group| group.exec.name == name)
    }

    fn get_output_pin_group_num(&self) -> usize {
        self.get_output_pin_group().len()
    }
}
