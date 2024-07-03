use bevy::prelude::*;

use super::{
    blackboard::EffectValue,
    node::{
        pin::{EffectNodeExec, EffectNodeSlot},
        EffectNodeId,
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Hash)]
pub struct EffectNodeExecPin {
    pub node_id: EffectNodeId,
    pub exec: EffectNodeExec,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Hash)]
pub struct EffectNodeSlotPin {
    pub node_id: EffectNodeId,
    pub slot: EffectNodeSlot,
}

#[derive(Debug, PartialEq, Clone, Reflect)]
pub enum EffectNodeSlotValue {
    Value(EffectValue),
    Ref(EffectNodeSlotPin),
}

impl From<EffectValue> for EffectNodeSlotValue {
    fn from(value: EffectValue) -> Self {
        Self::Value(value)
    }
}

impl From<EffectNodeSlotPin> for EffectNodeSlotValue {
    fn from(value: EffectNodeSlotPin) -> Self {
        Self::Ref(value)
    }
}
