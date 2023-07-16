use std::backtrace::Backtrace;

use bevy::{
    prelude::{info, Component, UVec3, Vec3},
    utils::HashMap,
};

use crate::terrain::isosurface::surface::shape_surface::ShapeSurface;

use super::sample_data::SampleData;

#[derive(Debug, Component)]
pub struct SurfaceSampler {
    /// UVec3 index size
    sample_data: SampleData<f32>,

    sample_pos: HashMap<UVec3, Vec3>,

    /// voxel size
    pub world_offset: Vec3,

    pub voxel_size: Vec3,
}

impl SurfaceSampler {
    pub fn new(world_offset: Vec3, voxel_size: Vec3, voxel_num: UVec3) -> SurfaceSampler {
        Self {
            sample_data: SampleData::new(voxel_num + UVec3::ONE),
            sample_pos: HashMap::default(),
            world_offset,
            voxel_size,
        }
    }

    pub fn get_sample_size(&self) -> UVec3 {
        self.sample_data.get_size()
    }

    pub fn set_sample_data(&mut self, sample_data: Vec<f32>) {
        self.sample_data.set_all_values(sample_data)
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
        &self,
        vertex_address: UVec3,
        _shape_surface: &ShapeSurface,
    ) -> f32 {
        // if self.sample_data.get_data_index(vertex_address) >= 4096 {
        //     info!("error vertex_address: {:?}", vertex_address);
        //     println!("Custom backtrace: {}", Backtrace::force_capture());
        // }
        self.sample_data.get_value(vertex_address)
    }

    /// todo: cache get values.
    pub fn get_value_from_vertex_offset(
        &self,
        vertex_address: UVec3,
        vertex_offset: Vec3,
        shape_surface: &ShapeSurface,
    ) -> f32 {
        let vertex_pos = vertex_address.as_vec3() + vertex_offset;
        self.get_value_from_pos(vertex_pos, shape_surface)
        // let vertex_alpha = vertex_offset / self.voxel_size;
        // let abs_vertex_alpha = vertex_alpha.abs();
        // assert!(
        //     abs_vertex_alpha.x >= 0.0
        //         && abs_vertex_alpha.x <= 1.0
        //         && abs_vertex_alpha.y >= 0.0
        //         && abs_vertex_alpha.y <= 1.0
        //         && abs_vertex_alpha.z >= 0.0
        //         && abs_vertex_alpha.z <= 1.0
        // );
        //
        // let vertex_address_next = vertex_address + vertex_alpha.ceil().as_uvec3();
        // let value = self.get_value_from_vertex_address(vertex_address, shape_surface);
        // let value_next = self.get_value_from_vertex_address(vertex_address_next, shape_surface);
        //
        // value
        //     + (value_next - value) * (abs_vertex_alpha.x + abs_vertex_alpha.y + abs_vertex_alpha.z)
        //         / 3.0
    }

    pub fn get_value_from_pos(&self, vertex_pos: Vec3, shape_surface: &ShapeSurface) -> f32 {
        let pos = self.world_offset + vertex_pos * self.voxel_size + shape_surface.iso_level;
        shape_surface.get_value_from_vec(pos)
    }
}
