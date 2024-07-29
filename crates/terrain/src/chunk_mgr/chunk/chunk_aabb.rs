use std::ops::{Deref, DerefMut};

use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    prelude::Component,
    reflect::Reflect,
    render::extract_component::ExtractComponent,
};

#[derive(Debug, Component, Reflect, Clone, ExtractComponent, Copy)]
pub struct TerrainChunkAabb(pub Aabb3d);

impl Default for TerrainChunkAabb {
    fn default() -> Self {
        Self(Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO))
    }
}

impl Deref for TerrainChunkAabb {
    type Target = Aabb3d;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TerrainChunkAabb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
