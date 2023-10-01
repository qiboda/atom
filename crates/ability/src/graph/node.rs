use std::any::TypeId;

use bevy::{
    prelude::{Commands, Component, Entity, EventWriter},
    reflect::reflect_trait,
    utils::Uuid,
};

use super::{context::EffectGraphContext, event::EffectEvent};

pub trait EffectNode {
    fn start(&mut self, start_context: EffectNodeStartContext);

    fn abort(&mut self, abort_context: EffectNodeAbortContext);

    fn pause(&mut self, pause_context: EffectNodePauseContext) {
        *pause_context.node_tick_state = EffectNodeTickState::Paused;
    }

    fn resume(&mut self, resume_context: EffectNodeResumeContext) {
        *resume_context.node_tick_state = EffectNodeTickState::Ticked;
    }
}

pub struct EffectNodeStartContext<'a, 'w: 'a, 's: 'a, 'e: 'a> {
    pub commands: &'a mut Commands<'w, 's>,
    pub node_entity: Entity,
    pub node_uuid: &'a EffectNodeUuid,
    pub node_tick_state: &'a mut EffectNodeTickState,
    pub node_state: &'a mut EffectNodeExecuteState,
    pub graph_context: &'a mut EffectGraphContext,
    pub event_writer: &'a mut EventWriter<'e, EffectEvent>,
}

pub struct EffectNodePauseContext<'a> {
    pub node_entity: Entity,
    pub node_uuid: &'a EffectNodeUuid,
    pub node_tick_state: &'a mut EffectNodeTickState,
    pub node_state: &'a mut EffectNodeExecuteState,
    pub graph_context: &'a mut EffectGraphContext,
}

pub struct EffectNodeResumeContext<'a> {
    pub node_entity: Entity,
    pub node_uuid: &'a EffectNodeUuid,
    pub node_tick_state: &'a mut EffectNodeTickState,
    pub node_state: &'a mut EffectNodeExecuteState,
    pub graph_context: &'a mut EffectGraphContext,
}

pub struct EffectNodeAbortContext<'a> {
    pub node_entity: Entity,
    pub node_uuid: &'a EffectNodeUuid,
    pub node_tick_state: &'a mut EffectNodeTickState,
    pub node_state: &'a mut EffectNodeExecuteState,
    pub graph_context: &'a mut EffectGraphContext,
}

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
