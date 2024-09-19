pub mod direction_projectile;

use bevy::prelude::*;

use super::{
    hit::ProjectileHitCount,
    lifetime::{
        ProjectileDestroyOpportunityOr, ProjectileDestroySelf, ProjectileLifetime, ProjectileState,
    },
    Projectile,
};

#[derive(Debug, Default, Bundle)]
pub struct ProjectileBaseBundle {
    pub projectile: Projectile,

    pub hit_count: ProjectileHitCount,
    pub destroy_opportunity: ProjectileDestroyOpportunityOr,
    pub destroy_owner: ProjectileDestroySelf,
    pub state: ProjectileState,
    pub lifetime: ProjectileLifetime,
}
