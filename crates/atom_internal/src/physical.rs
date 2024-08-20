use avian3d::prelude::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum PhysicalCollisionLayer {
    Terrain, // Layer 0
    Enemy,   // Layer 1
    Player,  // Layer 2
}
