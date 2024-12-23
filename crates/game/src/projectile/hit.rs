use avian3d::prelude::CollisionStarted;
use bevy::prelude::*;

use super::{lifetime::ProjectileState, Projectile};

// 击中次数
#[derive(Debug, Component, Default)]
pub struct ProjectileHitCount {
    pub count: usize,
    // time from every hit
    pub time_after_hit: Vec<f32>,
}

#[derive(Debug, Event)]
pub struct ProjectileHitEvent {
    pub projectile: Entity,
    pub target: Entity,
}

pub fn trigger_projectile_hit(
    trigger: Trigger<ProjectileHitEvent>,
    mut query: Query<&mut ProjectileHitCount, With<Projectile>>,
) {
    let event = trigger.event();
    let Ok(mut hit_count) = query.get_mut(event.projectile) else {
        return;
    };

    hit_count.count += 1;
    hit_count.time_after_hit.push(0.0);
}

pub fn update_hit_time(
    mut query: Query<&mut ProjectileHitCount, With<Projectile>>,
    time: Res<Time<Fixed>>,
) {
    for mut hit_count in query.iter_mut() {
        for time_from_hit in hit_count.time_after_hit.iter_mut() {
            *time_from_hit += time.delta_secs();
        }
    }
}

pub fn read_collision_start_event(
    mut commands: Commands,
    mut events: EventReader<CollisionStarted>,
    query: Query<&ProjectileState, With<Projectile>>,
) {
    for event in events.read() {
        if let Ok(state) = query.get(event.0) {
            if *state == ProjectileState::Running {
                commands.trigger_targets(
                    ProjectileHitEvent {
                        projectile: event.0,
                        target: event.1,
                    },
                    event.0,
                );
            }
        }

        if let Ok(state) = query.get(event.1) {
            if *state == ProjectileState::Running {
                commands.trigger_targets(
                    ProjectileHitEvent {
                        projectile: event.1,
                        target: event.0,
                    },
                    event.1,
                );
            }
        }
    }
}
