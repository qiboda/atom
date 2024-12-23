use avian3d::{
    collision::Collider,
    prelude::{LockedAxes, RigidBody},
};
use bevy::{
    core::Name,
    math::Vec3,
    prelude::{Bundle, Transform, Visibility},
};
use bevy_tnua::prelude::TnuaController;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

pub const UNIT_RADIUS: f32 = 0.5;
pub const UNIT_HEIGHT: f32 = 2.0;
pub const UNIT_DESIRED_SPEED: f32 = 2.0;
pub const UNIT_MAX_SPEED: f32 = 2.0;

#[derive(Bundle)]
pub struct ClientUnitBundle {
    // ability_subsystem: AbilitySubsystemBundle,
    // attribute_set
    // animation
    pub name: Name,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_locked_axes: LockedAxes,

    pub transform: Transform,
    pub visibility: Visibility,

    pub tnua_controller: TnuaController,
    pub tuna_sensor_shape: TnuaAvian3dSensorShape,
}

impl Default for ClientUnitBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Unit"),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(UNIT_RADIUS, UNIT_HEIGHT),
            collider_locked_axes: LockedAxes::ROTATION_LOCKED,
            tnua_controller: Default::default(),
            tuna_sensor_shape: TnuaAvian3dSensorShape(Collider::capsule(
                UNIT_RADIUS * 0.95,
                UNIT_HEIGHT * 0.95,
            )),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            visibility: Visibility::Visible,
        }
    }
}

impl ClientUnitBundle {
    pub fn new(radius: f32, length: f32) -> Self {
        Self {
            tuna_sensor_shape: TnuaAvian3dSensorShape(Collider::capsule(radius, length)),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Default, Debug)]
pub struct ServerUnitBundle {}
