use bevy::{
    prelude::{Component, Mesh, UVec3, Vec3, Vec4},
    render::render_resource::PrimitiveTopology,
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

impl From<&SurfaceSampler> for Mesh {
    fn from(value: &SurfaceSampler) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::PointList);

        let mut position = vec![];
        let mut color = vec![];
        let sample_size = value.get_sample_size();
        for x in 0..sample_size.x {
            for y in 0..sample_size.y {
                for z in 0..sample_size.z {
                    let pos_value = value.sample_data.get_value(UVec3::new(x, y, z));
                    let pos =
                        value.world_offset + value.voxel_size * UVec3::new(x, y, z).as_vec3() + 0.0;
                    if pos_value < 0.0 {
                        position.push(pos);
                        color.push(Vec4::new(0.0, 255.0, 0.0, 255.0));
                    } else if pos_value == 0.0 {
                        position.push(pos);
                        color.push(Vec4::new(255.0, 0.0, 0.0, 255.0));
                    }
                }
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);

        // mesh.insert_attribute(
        //     Mesh::ATTRIBUTE_POSITION,
        //     value
        //         .sample_pos
        //         .into_iter()
        //         .map(|(_, pos)| pos)
        //         .collect::<Vec<Vec3>>(),
        // );
        //
        mesh
    }
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

        let pos = self.world_offset
            + self.voxel_size * vertex_address.as_vec3()
            + shape_surface.iso_level;
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
        let vertex_pos = vertex_address.as_vec3();
        let pos = self.world_offset
            + vertex_pos * self.voxel_size
            + shape_surface.iso_level
            + vertex_offset;
        shape_surface.get_value_from_vec(pos)
    }

    pub fn get_value_from_pos(&self, vertex_pos: Vec3, shape_surface: &ShapeSurface) -> f32 {
        shape_surface.get_value_from_vec(vertex_pos)
    }
}
