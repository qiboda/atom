use std::any::TypeId;

use bevy::{
    prelude::{Commands, Component, Entity, EventWriter},
    reflect::reflect_trait,
    utils::Uuid,
};

use super::{context::EffectGraphContext, event::EffectEvent};

pub trait EffectNode {
    fn start(
        &mut self,
        commands: &mut Commands,
        node_entity: Entity,
        node_uuid: &EffectNodeUuid,
        node_state: &mut EffectNodeState,
        graph_context: &mut EffectGraphContext,
        event_writer: &mut EventWriter<EffectEvent>,
    );

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
    Default,
    Paused,
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
