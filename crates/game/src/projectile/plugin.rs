use bevy::app::Plugin;

use super::event::{ProjectileEndEvent, ProjectileHitEvent, ProjectileStartEvent};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ProjectileStartEvent>()
            .add_event::<ProjectileEndEvent>()
            .add_event::<ProjectileHitEvent>();
    }
}
