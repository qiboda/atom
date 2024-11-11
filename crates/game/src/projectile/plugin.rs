use bevy::app::{FixedPostUpdate, FixedPreUpdate, Plugin};

use super::{
    hit::{trigger_projectile_hit, update_hit_time, ProjectileHitEvent},
    lifetime::{
        destroy_projectile, trigger_projectile_end, trigger_projectile_start, update_lifetime,
        ProjectileEndEvent, ProjectileStartEvent,
    },
};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ProjectileStartEvent>()
            .add_event::<ProjectileEndEvent>()
            .add_event::<ProjectileHitEvent>()
            .observe(trigger_projectile_start)
            .observe(trigger_projectile_end)
            .observe(trigger_projectile_hit)
            .add_systems(FixedPreUpdate, (update_lifetime, update_hit_time))
            .add_systems(FixedPostUpdate, destroy_projectile);
    }
}
