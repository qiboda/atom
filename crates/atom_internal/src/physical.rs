use avian3d::prelude::PhysicsLayer;

#[derive(PhysicsLayer, Default)]
pub enum PhysicalCollisionLayer {
    #[default]
    Terrain, // Layer 0
    Enemy,   // Layer 1
    Player,  // Layer 2
}
