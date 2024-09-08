use avian3d::{
    collision::Collider,
    prelude::{LockedAxes, RigidBody},
};
use bevy::{
    asset::Handle,
    core::Name,
    pbr::StandardMaterial,
    prelude::{Bundle, Mesh},
};
use bevy_tnua::controller::TnuaControllerBundle;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

use crate::network::bevy_bundle::{ClientSpatialBundle, ServerSpatialBundle};

#[derive(Bundle)]
pub struct ClientUnitBundle {
    // ability_subsystem: AbilitySubsystemBundle,
    // attribute_set
    // mesh
    // visibility
    // transform?
    // animation
    pub name: Name,

    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,

    pub spatial_bundle: ClientSpatialBundle,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_locked_axes: LockedAxes,
    pub tnua_controller: TnuaControllerBundle,
    pub tuna_sensor_shape: TnuaAvian3dSensorShape,
}

impl Default for ClientUnitBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(0.5, 2.0),
            name: Name::new("Unit"),
            collider_locked_axes: LockedAxes::ROTATION_LOCKED,

            tnua_controller: TnuaControllerBundle::default(),
            tuna_sensor_shape: TnuaAvian3dSensorShape(Collider::capsule(0.5, 2.0)),
            mesh: Handle::default(),
            material: Handle::default(),
            spatial_bundle: ClientSpatialBundle::default(),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ServerUnitBundle {
    pub spatial_bundle: ServerSpatialBundle,
}
