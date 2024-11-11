pub mod implement;
pub mod movement;
pub mod plugin;
pub mod lifetime;
pub mod hit;
pub mod effect;

use bevy::prelude::Component;

#[derive(Debug, Default, Component)]
pub struct Projectile;
