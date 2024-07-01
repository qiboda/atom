pub mod hatred;
pub mod sensing;

use bevy::prelude::*;
use hatred::Hatred;

#[derive(Debug, Component, Default)]
pub struct AttackTarget(pub Option<Entity>);

#[derive(Debug, Component, Default)]
pub struct HealingTarget(pub Option<Entity>);

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TargetsSystemSet {
    Sensing,
    Hatred,
    UpdateTarget,
}

#[derive(Debug, Default)]
pub struct TargetsPlugin;

impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (refresh_attack_target, refresh_healing_target).in_set(TargetsSystemSet::UpdateTarget),
        );
    }
}

fn refresh_attack_target(mut query: Query<(&mut AttackTarget, &Hatred)>) {
    for (mut target, hatred) in query.iter_mut() {
        target.0 = hatred.get_max_hatred_value_entity();
    }
}

fn refresh_healing_target(mut query: Query<(&mut HealingTarget, &Hatred)>) {
    for (mut target, hatred) in query.iter_mut() {
        target.0 = hatred.get_max_friendly_value_entity();
    }
}
