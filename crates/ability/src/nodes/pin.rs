use std::borrow::Cow;

use bevy::{
    prelude::{Component, Entity},
    reflect::Reflect,
};

#[derive(Debug, Clone)]
pub struct EffectNodePinRef {
    node: Entity,
    pin_name: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub enum EffectNodePinValue {
    // children nodes
    Exec(Vec<Entity>),
    /// set value directly
    Value(dyn Reflect),
    /// Reference to another pin
    Reference(EffectNodePinRef),
}

/// T is input pin type
#[derive(Debug, Clone)]
pub struct EffectNodeInput<T: Reflect> {
    pin_name: Cow<'static, str>,
    pin_value: EffectNodePinValue,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Component, Clone)]
pub struct EffectNodeInputs {
    pub inputs: Vec<EffectNodeInput<dyn Reflect>>,
}

#[derive(Debug, Clone)]
pub struct EffectNodeOutput<T: Reflect> {
    pin_name: Cow<'static, str>,
    pin_value: EffectNodePinValue,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Component, Clone)]
pub struct EffectNodeOutputs {
    pub inputs: Vec<EffectNodeOutput<dyn Reflect>>,
}
