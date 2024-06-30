use bevy::prelude::*;

use super::sensing::hatred::Hatred;

#[derive(Debug, Component, Default)]
pub struct AttackTarget(pub Option<Entity>);

#[derive(Debug, Component, Default)]
pub struct HealingTarget(pub Option<Entity>);

#[derive(Debug, Default)]
pub struct TargetsPlugin;

impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, refresh_attack_target);
    }
}

fn refresh_attack_target(mut query: Query<(&mut AttackTarget, &Hatred)>) {
    for (mut target, hatred) in query.iter_mut() {
        target.0 = hatred.get_max_hatred_value_entity();
    }
}
