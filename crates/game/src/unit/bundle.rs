use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::{
    core::Name,
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::{Bundle, SpatialBundle},
};

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
}

impl Default for UnitBundle {
    fn default() -> Self {
        Self {
            mat_mesh: Default::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(0.5, 2.0),
            name: Name::default(),
        }
    }
}
