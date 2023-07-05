use bevy::{
    prelude::{Component, UVec3, Vec3},
    utils::HashMap,
};

use crate::isosurface::surface::shape_surface::ShapeSurface;

#[derive(Debug, Component)]
pub struct SurfaceSampler {
    /// UVec3 index size
    sample_data: HashMap<UVec3, f32>,

    sample_pos: HashMap<UVec3, Vec3>,

    /// voxel size
    pub world_offset: Vec3,

    pub voxel_size: Vec3,
}

impl SurfaceSampler {
    pub fn new(world_offset: Vec3, voxel_size: Vec3) -> SurfaceSampler {
        Self {
            sample_data: HashMap::default(),
            sample_pos: HashMap::default(),
            world_offset,
            voxel_size,
        }
    }
}

impl SurfaceSampler {
    pub fn get_pos_from_vertex_address(
        &mut self,
        vertex_address: UVec3,
        shape_surface: &ShapeSurface,
    ) -> Vec3 {
        if let Some(value) = self.sample_pos.get(&vertex_address) {
            return *value;
        }

        let pos = self.voxel_size * vertex_address.as_vec3() + shape_surface.iso_level;
        self.sample_pos.insert(vertex_address, pos);
        pos
    }
}

impl SurfaceSampler {
    pub fn get_value_from_vertex_address(
        &mut self,
        vertex_address: UVec3,
        shape_surface: &ShapeSurface,
    ) -> f32 {
        if let Some(value) = self.sample_data.get(&vertex_address) {
            return *value;
        }

        let pos = self.voxel_size * vertex_address.as_vec3() + shape_surface.iso_level;
        let value = shape_surface.get_value_from_vec(pos);
        self.sample_data.insert(vertex_address, value);
        value
    }

    /// todo: cache get values.
    pub fn get_value_from_vertex_offset(
        &self,
        vertex_address: UVec3,
        vertex_offset: Vec3,
        shape_surface: &ShapeSurface,
    ) -> f32 {
        let pos =
            self.voxel_size * vertex_address.as_vec3() + shape_surface.iso_level + vertex_offset;
        shape_surface.get_value_from_vec(pos)
    }

    pub fn get_value_from_pos(&self, pos: Vec3, shape_surface: &ShapeSurface) -> f32 {
        shape_surface.get_value_from_vec(pos)
    }
}
