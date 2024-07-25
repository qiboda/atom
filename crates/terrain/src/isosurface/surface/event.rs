use bevy::{math::bounding::Aabb3d, prelude::*};

#[derive(Event, Debug, Clone, Copy)]
pub struct CSGOperationEndEvent {
    pub aabb: Aabb3d,
}
