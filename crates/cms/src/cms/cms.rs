use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    octree::{def::MAX_OCTREE_RES, edge_block::VertexIndices, vertex::Vertex},
    sample::{sample_info::SampleInfo, sample_range_3d::SampleRange3D},
    surface::shape_surface::ShapeSurface,
};

use super::bundle::CMSBundle;

#[derive(Debug, Component, Default)]
pub struct CMSMeshInfo {
    pub vertices: Vec<Vertex>,
    pub vertex_index_data: SampleRange3D<VertexIndices>,
    pub snap_centro_id: bool,
}

pub fn cms_startup(commands: &mut Commands) {
    let mut sample_size = Vector3::new(0, 0, 0);
    sample_size.x = 2usize.pow(MAX_OCTREE_RES as u32) + 1;
    sample_size.y = sample_size.x;
    sample_size.z = sample_size.x;

    // todo: set container
    let container = Vector3::new((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));

    let offsets = Vector3::new(
        (container.x.1 - container.x.0) / (sample_size.x - 1) as f32,
        (container.y.1 - container.y.0) / (sample_size.y - 1) as f32,
        (container.z.1 - container.z.0) / (sample_size.z - 1) as f32,
    );

    commands.spawn(CMSBundle {
        cms_mesh_info: CMSMeshInfo {
            vertices: Vec::new(),
            vertex_index_data: SampleRange3D::new(container, sample_size),
            snap_centro_id: false,
        },
        sample_info: SampleInfo {
            samples_size: sample_size, // todo: remove
            sample_data: SampleRange3D::new(container, sample_size),
            offsets,
        },
    });
}

pub fn cms_initialize_sample_data(
    shape_surface: Res<ShapeSurface>,
    query: Query<&SampleInfo, Added<SampleInfo>>,
) {
    info!("CMS::new");

    for mut sample_info in query.iter_mut() {
        info!("CMS::sample size start");
        for i in 0..sample_info.samples_size.x {
            let x = sample_info.sample_data.get_pos_size().x.0
                + i as f32 * sample_info.offsets.x
                + shape_surface.iso_level;

            for j in 0..sample_info.samples_size.y {
                let y = sample_info.sample_data.get_pos_size().y.0
                    + j as f32 * sample_info.offsets.y
                    + shape_surface.iso_level;

                for k in 0..sample_info.samples_size.z {
                    let z = sample_info.sample_data.get_pos_size().z.0
                        + k as f32 * sample_info.offsets.z
                        + shape_surface.iso_level;

                    let value = shape_surface.get_value(x, y, z);
                    sample_info.sample_data.set_value(i, j, k, value);
                }
            }
        }
        info!("CMS::sample size end");
    }
}
