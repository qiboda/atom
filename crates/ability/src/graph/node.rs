use std::any::TypeId;

use bevy::{
    prelude::*,
    reflect::reflect_trait,
    utils::Uuid,
};

pub trait EffectNode {}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub enum EffectNodeTickState {
    #[default]
    Ticked,
    Paused,
}

#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq)]
pub enum EffectNodeExecuteState {
    #[default]
    Idle,
    Actived,
}

/// use for deserialize and serialize
#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Hash)]
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
