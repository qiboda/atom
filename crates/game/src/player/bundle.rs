use bevy::{pbr::{MaterialMeshBundle, StandardMaterial}, prelude::Bundle};

#[derive(Bundle, Default)]
pub struct UnitBundle {
    // ability_subsystem: AbilitySubsystemBundle,
    // attribute_set
    // mesh
    // visibility
    // transform?
    // material
    // animation
    // physical
    // collision
    mat_mesh: MaterialMeshBundle<StandardMaterial>
}



pub struct Player;
