use bevy::app::{App, Plugin, PreUpdate};
use big_brain::BigBrainPlugin;

pub mod follow_player;

pub struct AiBrainPlugin;

impl Plugin for AiBrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate))
            .add_plugins(follow_player::FollowPlayerThinkerPlugin);
    }
}
