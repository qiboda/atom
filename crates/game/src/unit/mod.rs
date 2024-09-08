use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod attr_set;
pub mod bundle;
pub mod player;

#[derive(Debug, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Player;

#[derive(Debug, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Npc;

#[derive(Debug, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Monster;

pub type UnitQueryFilter = Or<(With<Player>, With<Monster>, With<Npc>)>;
