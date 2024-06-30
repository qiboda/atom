use bevy::prelude::Component;

pub mod attr_set;
pub mod bundle;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct Npc;

#[derive(Debug, Component)]
pub struct Monster;
