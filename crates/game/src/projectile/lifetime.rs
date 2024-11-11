use bevy::prelude::*;

use super::{hit::ProjectileHitCount, Projectile};

#[derive(Debug, Event)]
pub struct ProjectileEndEvent {
    pub projectile: Entity,
}

#[derive(Debug, Event)]
pub struct ProjectileStartEvent {
    pub projectile: Entity,
}

pub fn trigger_projectile_start(
    trigger: Trigger<ProjectileStartEvent>,
    mut query: Query<&mut ProjectileState, With<Projectile>>,
) {
    let event = trigger.event();
    let Ok(mut state) = query.get_mut(event.projectile) else {
        return;
    };

    *state = ProjectileState::Running;
}

pub fn trigger_projectile_end(
    trigger: Trigger<ProjectileEndEvent>,
    mut query: Query<&mut ProjectileState, With<Projectile>>,
) {
    let event = trigger.event();
    let Ok(mut state) = query.get_mut(event.projectile) else {
        return;
    };

    *state = ProjectileState::End;
}

#[derive(Debug, Component)]
pub enum ProjectileDestroyOpportunity {
    // 击中次数后经过多长时间。
    Hit(usize, f32),
    // 到达目的地后经过多长时间。
    End(f32),
    // 经过多长时间后销毁。
    Time(f32),
}

#[derive(Debug, Default, Component, PartialEq, Eq)]
pub enum ProjectileState {
    #[default]
    Wait,
    Running,
    End,
}

impl ProjectileState {
    pub fn can_move(&self) -> bool {
        match self {
            ProjectileState::Wait => false,
            ProjectileState::Running => true,
            ProjectileState::End => true,
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct ProjectileDestroyOpportunityOr {
    pub opportunity_or: Vec<ProjectileDestroyOpportunity>,
}

impl Default for ProjectileDestroyOpportunity {
    fn default() -> Self {
        ProjectileDestroyOpportunity::Hit(1, 0.0)
    }
}

#[derive(Debug, Component, Default)]
pub enum ProjectileDestroySelf {
    #[default]
    This,
    Ref(Entity),
}

#[derive(Debug, Component, Default)]
pub struct ProjectileLifetime {
    lifetime: f32,
    time_after_end: f32,
}

pub fn update_lifetime(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut ProjectileLifetime, &ProjectileState), With<Projectile>>,
) {
    for (mut lifetime, state) in query.iter_mut() {
        match state {
            ProjectileState::Wait => continue,
            ProjectileState::Running => {
                lifetime.lifetime += time.delta_seconds();
            }
            ProjectileState::End => {
                lifetime.lifetime += time.delta_seconds();
                lifetime.time_after_end += time.delta_seconds();
            }
        }
    }
}

pub fn destroy_projectile(
    query: Query<
        (
            Entity,
            &ProjectileDestroyOpportunityOr,
            &ProjectileLifetime,
            &ProjectileDestroySelf,
            &ProjectileHitCount,
            &ProjectileState,
        ),
        With<Projectile>,
    >,
    mut commands: Commands,
) {
    for (entity, opportunity_or, lifetime, destroy_self, hit_count, state) in query.iter() {
        let mut to_destroy = false;
        for opportunity in opportunity_or.opportunity_or.iter() {
            match opportunity {
                ProjectileDestroyOpportunity::Hit(count, delay_time) => {
                    if hit_count.count >= *count && hit_count.time_after_hit[*count] >= *delay_time
                    {
                        to_destroy = true;
                    }
                }
                ProjectileDestroyOpportunity::End(time) => {
                    if *state == ProjectileState::End && lifetime.time_after_end >= *time {
                        to_destroy = true;
                    }
                }
                ProjectileDestroyOpportunity::Time(time) => {
                    if lifetime.lifetime >= *time {
                        to_destroy = true;
                    }
                }
            }
        }

        if to_destroy {
            match destroy_self {
                ProjectileDestroySelf::This => {
                    commands.entity(entity).despawn_recursive();
                }
                ProjectileDestroySelf::Ref(ref_entity) => {
                    commands.entity(*ref_entity).despawn_recursive();
                }
            }
        }
    }
}
