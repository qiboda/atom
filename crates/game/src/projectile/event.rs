use bevy::prelude::{Entity, Event};

#[derive(Debug, Event)]
pub struct ProjectileHitEvent {
    pub projectile: Entity,
    pub target: Entity,
}

#[derive(Debug, Event)]
pub struct ProjectileEndEvent {
    pub projectile: Entity,
}

#[derive(Debug, Event)]
pub struct ProjectileStartEvent {
    pub projectile: Entity,
}
