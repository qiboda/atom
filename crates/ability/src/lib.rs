/// ability crate
use bevy::{prelude::Component, reflect::Reflect};

pub mod attribute;
pub mod bundle;
pub mod effect;
pub mod graph;
pub mod stateset;

// only ability with this component
#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Ability;

// non-aiblity with this component, such as buff, debuff, etc.
#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Effect;

// todo: add ability plugin
