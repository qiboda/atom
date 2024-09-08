use std::ops::Not;

use aery::prelude::*;
use avian3d::prelude::PhysicsSet;
use bevy::prelude::*;

use crate::transform::TransformFreedom;

#[derive(Debug)]
pub struct TransformFollowPlugin;

impl Plugin for TransformFollowPlugin {
    fn build(&self, app: &mut App) {
        app.register_relation::<Following>()
            .observe(trigger_set_followed)
            .add_systems(
                PostUpdate,
                update_follow
                    .after(PhysicsSet::Sync)
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

#[derive(Component, Debug, Default)]
pub struct RelativeTransform(pub Transform);

#[derive(Component, Debug, Default)]
pub struct RelativeTransformFreedom(pub TransformFreedom);

/// make a entity to follow another entity.
#[derive(Relation, Debug, Default)]
pub struct Following;

#[allow(clippy::type_complexity)]
pub fn trigger_set_followed(
    trigger: Trigger<SetEvent<Following>>,
    mut commands: Commands,
    query: Query<
        (Has<RelativeTransform>, Has<RelativeTransformFreedom>),
        (
            Or<(
                Without<RelativeTransform>,
                Without<RelativeTransformFreedom>,
            )>,
        ),
    >,
) {
    let host = trigger.entity();

    if let Ok((relative_transform, relative_freedom)) = query.get(host) {
        if relative_transform.not() {
            commands.entity(host).insert(RelativeTransform::default());
            info!("insert RelativeTransform");
        }
        if relative_freedom.not() {
            commands
                .entity(host)
                .insert(RelativeTransformFreedom::default());
            info!("insert RelativeTransformFreedom");
        }
    }
}

pub fn update_follow(
    transform_query: Query<(&GlobalTransform, Relations<Following>)>,
    mut followed_query: Query<(
        &mut Transform,
        &RelativeTransform,
        Option<&RelativeTransformFreedom>,
    )>,
) {
    for (following_transform, rel) in transform_query.iter() {
        rel.join::<Following>(&mut followed_query).for_each(
            |(mut transform, rel_transform, rel_transform_freedom)| {
                *transform = rel_transform.0;
                let following_transform = following_transform.compute_transform();
                match rel_transform_freedom {
                    Some(rel_transform_freedom) => match &rel_transform_freedom.0 {
                        TransformFreedom::None => *transform = *transform * following_transform,
                        TransformFreedom::Lock(locked_freedom) => {
                            match &locked_freedom.locked_translation {
                                Some(locked_translation) => {
                                    if locked_translation.locked_x.not() {
                                        transform.translation.x +=
                                            following_transform.translation.x;
                                    }
                                    if locked_translation.locked_y.not() {
                                        transform.translation.y +=
                                            following_transform.translation.y;
                                    }
                                    if locked_translation.locked_z.not() {
                                        transform.translation.z +=
                                            following_transform.translation.z;
                                    }
                                }
                                None => {
                                    transform.translation += following_transform.translation;
                                }
                            }

                            match &locked_freedom.locked_rotation {
                                Some(locked_rotation) => {
                                    let following_eular =
                                        following_transform.rotation.to_euler(EulerRot::XYZ);
                                    // println!("{:?}", following_eular);
                                    if locked_rotation.locked_pitch.not() {
                                        transform.rotate_z(following_eular.2);
                                    }
                                    if locked_rotation.locked_yaw.not() {
                                        transform.rotate_y(following_eular.1);
                                    }
                                    if locked_rotation.locked_roll.not() {
                                        transform.rotate_x(following_eular.0);
                                    }
                                }
                                None => transform.rotation *= following_transform.rotation,
                            }

                            match &locked_freedom.locked_scale {
                                Some(locked_scale) => {
                                    if locked_scale.locked_x.not() {
                                        transform.scale.x *= following_transform.scale.x;
                                    }
                                    if locked_scale.locked_y.not() {
                                        transform.scale.y *= following_transform.scale.y;
                                    }
                                    if locked_scale.locked_z.not() {
                                        transform.scale.z *= following_transform.scale.z;
                                    }
                                }
                                None => {
                                    transform.scale += following_transform.scale;
                                }
                            }
                        }
                    },
                    None => {
                        *transform = *transform * following_transform;
                    }
                }
            },
        );
    }
}
