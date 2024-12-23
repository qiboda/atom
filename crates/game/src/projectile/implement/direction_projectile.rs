use std::ops::Not;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::projectile::{
    lifetime::{ProjectileStartEvent, ProjectileState},
    movement::SpeedVariant,
    Projectile,
};

use super::ProjectileBaseBundle;

pub struct ProjectileLineSpeedPlugin;

impl Plugin for ProjectileLineSpeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(trigger_projectile_start)
            .add_systems(FixedUpdate, update_projectile_movement);
    }
}

#[derive(Bundle, Debug)]
pub struct ProjectileLineSpeedBundle {
    pub projectile_base: ProjectileBaseBundle,
    pub movement: ProjectileLineMovement,
}

#[derive(Component, Debug)]
pub struct ProjectileLineMovement {
    pub speed: SpeedVariant,
    pub direction: Dir3,
}

fn trigger_projectile_start(
    trigger: Trigger<ProjectileStartEvent>,
    mut query: Query<
        (&ProjectileLineMovement, &mut Transform, &mut LinearVelocity),
        With<Projectile>,
    >,
) {
    let event = trigger.event();

    if let Ok((movement, mut transform, mut linear_velocity)) = query.get_mut(event.projectile) {
        let direction = movement.direction;
        transform.look_to(direction, Vec3::Y);

        let speed = &movement.speed;
        match speed {
            SpeedVariant::Constant(speed) => {
                linear_velocity.0 = direction * *speed;
            }
            SpeedVariant::Derivative(speed, _acceleration) => {
                linear_velocity.0 = direction * *speed;
            }
        }
    }
}

fn update_projectile_movement(
    time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &ProjectileLineMovement,
            &ProjectileState,
            &mut LinearVelocity,
        ),
        With<Projectile>,
    >,
) {
    for (movement, state, mut linear_velocity) in query.iter_mut() {
        if state.can_move().not() {
            continue;
        }

        match movement.speed {
            SpeedVariant::Constant(_speed) => {}
            SpeedVariant::Derivative(_, acceleration) => {
                let delta_seconds = time.delta_secs();
                let direction = movement.direction;

                linear_velocity.0 += direction * acceleration * delta_seconds;
            }
        }
    }
}
