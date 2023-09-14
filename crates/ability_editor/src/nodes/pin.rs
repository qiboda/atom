use std::borrow::Cow;

use bevy::prelude::{Component, Entity};

use super::blackboard::EffectValue;

#[derive(Debug, Default)]
pub enum EffectNodePinExecType {
    #[default]
    Sync,
    Async,
}

#[derive(Debug, Clone)]
pub struct EffectNodePinRef {
    pub node: Entity,
    pub pin_name: Cow<'static, str>,
}

impl Default for EffectNodePinRef {
    fn default() -> Self {
        Self {
            node: Entity::PLACEHOLDER,
            pin_name: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct EffectNodePinExec {
    pub pin_type: EffectNodePinExecType,
    pub children: Option<Vec<Entity>>,
}

#[derive(Debug)]
pub enum EffectNodePinValue {
    Set(EffectValue),
    Ref(Option<EffectNodePinRef>),
}

#[derive(Debug)]
pub enum EffectNodePinType {
    // children nodes
    Exec(EffectNodePinExec),
    /// set value directly
    Value(EffectNodePinValue),
}

impl Default for EffectNodePinType {
    fn default() -> Self {
        Self::Exec(EffectNodePinExec::default())
    }
}

/// T is input pin type
#[derive(Debug, Default)]
pub struct EffectNodePin {
    pub pin_name: Cow<'static, str>,
    pub pin_type: EffectNodePinType,
}

/// T is input pin type
#[derive(Debug, Default)]
pub struct EffectNodePinGroup {
    pub pin_group: Vec<EffectNodePin>,
}

#[derive(Debug, Default, Component)]
pub struct EffectNodeInput {
    pub exec_group: Vec<EffectNodePinGroup>,
}

#[derive(Debug, Default, Component)]
pub struct EffectNodeOutput {
    pub exec_group: Vec<EffectNodePinGroup>,
}

impl EffectNodeOutput {
    pub fn find_pin(&self, pin_name: &str) -> Option<&EffectNodePin> {
        self.exec_group.iter().find_map(|pin_group| {
            pin_group
                .pin_group
                .iter()
                .find(|pin| pin.pin_name == pin_name)
        })
    }

    pub fn find_pin_mut(&mut self, pin_name: &str) -> Option<&mut EffectNodePin> {
        self.exec_group.iter_mut().find_map(|pin_group| {
            pin_group
                .pin_group
                .iter_mut()
                .find(|pin| pin.pin_name == pin_name)
        })
    }
}
