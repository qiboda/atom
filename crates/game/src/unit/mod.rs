use bevy::prelude::*;

pub mod attr_set;
pub mod bundle;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct Npc;

#[derive(Debug, Component)]
pub struct Monster;

pub type UnitQueryFilter = Or<(With<Player>, With<Monster>, With<Npc>)>;
