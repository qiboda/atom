use avian3d::{
    collision::Collider,
    prelude::{LockedAxes, RigidBody},
};
use bevy::{
    core::Name,
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::Bundle,
};
use bevy_tnua::controller::TnuaControllerBundle;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

#[derive(Bundle)]
pub struct UnitBundle {
    // ability_subsystem: AbilitySubsystemBundle,
    // attribute_set
    // mesh
    // visibility
    // transform?
    // animation
    pub name: Name,
    pub mat_mesh: MaterialMeshBundle<StandardMaterial>,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_locked_axes: LockedAxes,
    pub tnua_controller: TnuaControllerBundle,
    pub tuna_sensor_shape: TnuaAvian3dSensorShape,
}

impl Default for UnitBundle {
    fn default() -> Self {
        Self {
            mat_mesh: Default::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(0.5, 2.0),
            name: Name::new("Unit"),
            collider_locked_axes: LockedAxes::ROTATION_LOCKED,
            tnua_controller: TnuaControllerBundle::default(),
            tuna_sensor_shape: TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        }
    }
}
