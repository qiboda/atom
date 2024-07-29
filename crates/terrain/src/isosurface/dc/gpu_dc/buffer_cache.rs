use bevy::{
    math::bounding::Aabb3d,
    prelude::*,
    render::{
        render_resource::{BufferVec, CommandEncoder, ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
    },
    utils::HashMap,
};
use wgpu_types::BufferUsages;

use crate::{
    chunk_mgr::chunk::state::{TerrainChunkAddress, TerrainChunkSeamLod},
    lod::lod_octree::LodOctreeLevelType,
    setting::TerrainSetting,
    tables::SubNodeIndex,
};

use super::staged_buffer;

#[derive(ShaderType, Default, Clone, Copy, Debug)]
pub struct VoxelEdgeCrossPoint {
    // w is exist or not
    pub cross_pos: Vec4,
    // xyz is normal, w is material_index
    pub normal_material_index: Vec4,
}

#[derive(ShaderType)]
pub struct TerrainChunkInfo {
    // aabb的min 和 w作为chunk的size
    pub chunk_min_location_size: Vec4,
    // unit: meter
    pub voxel_size: f32,
    // unit: meter
    pub voxel_num: u32,
    // qef_threshold < 0 => 不使用qef
    pub qef_threshold: f32,
    pub qef_stddev: f32,
}

pub struct TerrainChunkMainBuffers {
    pub terrain_chunk_info_buffer: UniformBuffer<TerrainChunkInfo>,
    pub voxel_vertex_values_buffer: BufferVec<f32>,
    pub voxel_cross_points_buffer: BufferVec<VoxelEdgeCrossPoint>,

    pub mesh_vertex_map_buffer: BufferVec<u32>,
    pub mesh_vertex_location_buffer: staged_buffer::StagedBufferVec<Vec4>,
    pub mesh_vertex_normal_buffer: staged_buffer::StagedBufferVec<Vec4>,
    pub mesh_vertex_materials_buffer: staged_buffer::StagedBufferVec<u32>,
    pub mesh_indices_buffer: staged_buffer::StagedBufferVec<u32>,

    pub mesh_vertices_num_buffer: staged_buffer::StagedBuffer<u32>,
    pub mesh_indices_num_buffer: staged_buffer::StagedBuffer<u32>,
}

pub struct TerrainChunkMainBufferCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub render_queue: &'a RenderQueue,
    pub terrain_chunk_aabb: Aabb3d,
    pub terrain_chunk_address: TerrainChunkAddress,
    pub terrain_setting: &'a TerrainSetting,
}

impl TerrainChunkMainBuffers {
    pub fn update_buffers_reuse_info(&mut self, context: TerrainChunkMainBufferCreateContext) {
        let level = context.terrain_chunk_address.0.level();
        let chunk_size = context.terrain_setting.get_chunk_size(level);
        let voxel_size = context.terrain_setting.get_voxel_size(level);
        let voxel_num = context.terrain_setting.get_voxel_num_in_chunk();

        let chunk_min = context.terrain_chunk_aabb.min;

        self.terrain_chunk_info_buffer.set(TerrainChunkInfo {
            chunk_min_location_size: Vec4::new(chunk_min.x, chunk_min.y, chunk_min.z, chunk_size),
            voxel_size,
            voxel_num: voxel_num as u32,
            qef_threshold: context.terrain_setting.qef_solver_threshold,
            qef_stddev: context.terrain_setting.qef_stddev,
        });
        self.terrain_chunk_info_buffer
            .write_buffer(context.render_device, context.render_queue);

        self.mesh_vertices_num_buffer.set_value(0);
        self.mesh_vertices_num_buffer
            .write_buffer(context.render_device, context.render_queue);

        self.mesh_indices_num_buffer.set_value(0);
        self.mesh_indices_num_buffer
            .write_buffer(context.render_device, context.render_queue);
    }

    pub fn create_buffers(context: TerrainChunkMainBufferCreateContext) -> TerrainChunkMainBuffers {
        let voxel_num = context.terrain_setting.get_voxel_num_in_chunk();
        let voxel_vertex_num = voxel_num + 1;
        let total_voxel_num = voxel_num * voxel_num * voxel_num;
        let vertex_num = voxel_num * voxel_num * 7;
        let total_voxel_vertex_num = voxel_vertex_num * voxel_vertex_num * voxel_vertex_num;

        let chunk_min = context.terrain_chunk_aabb.min;

        let terrain_chunk_info_buffer = {
            let level = context.terrain_chunk_address.0.level();
            let chunk_size = context.terrain_setting.get_chunk_size(level);
            let voxel_size = context.terrain_setting.get_voxel_size(level);
            let mut chunk_info_uniform = UniformBuffer::from(TerrainChunkInfo {
                chunk_min_location_size: Vec4::new(
                    chunk_min.x,
                    chunk_min.y,
                    chunk_min.z,
                    chunk_size,
                ),
                voxel_size,
                voxel_num: voxel_num as u32,
                qef_threshold: context.terrain_setting.qef_solver_threshold,
                qef_stddev: context.terrain_setting.qef_stddev,
            });
            chunk_info_uniform.set_label(Some("terrain chunk info uniform buffer"));
            chunk_info_uniform.write_buffer(context.render_device, context.render_queue);
            chunk_info_uniform
        };

        let voxel_vertex_values_buffer = {
            let mut vertex_buffer = BufferVec::<f32>::new(BufferUsages::STORAGE);
            vertex_buffer.set_label(Some("terrain chunk voxel vertex values buffer"));
            vertex_buffer.reserve(total_voxel_vertex_num, context.render_device);
            vertex_buffer
        };

        let voxel_cross_points_buffer = {
            let mut vertex_buffer = BufferVec::<VoxelEdgeCrossPoint>::new(BufferUsages::STORAGE);
            vertex_buffer.set_label(Some("terrain chunk voxel cross point buffer"));
            vertex_buffer.reserve(total_voxel_vertex_num * 3, context.render_device);
            vertex_buffer
        };

        let mesh_vertex_map_buffer = {
            let mut vertex_buffer = BufferVec::<u32>::new(BufferUsages::STORAGE);
            vertex_buffer.set_label(Some("terrain chunk mesh vertex map buffer"));
            vertex_buffer.reserve(total_voxel_num, context.render_device);
            vertex_buffer
        };

        let mesh_vertex_location_buffer = staged_buffer::StagedBufferVec::<Vec4>::create_buffer(
            context.render_device,
            &format!("terrain chunk vertex location buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            vertex_num,
        );

        let mesh_vertex_normal_buffer = staged_buffer::StagedBufferVec::<Vec4>::create_buffer(
            context.render_device,
            &format!("terrain chunk vertex normal buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            vertex_num,
        );

        let mesh_vertex_materials_buffer = staged_buffer::StagedBufferVec::<u32>::create_buffer(
            context.render_device,
            &format!("terrain chunk vertex material buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            vertex_num,
        );

        let mesh_indices_buffer = staged_buffer::StagedBufferVec::<u32>::create_buffer(
            context.render_device,
            &format!("terrain chunk indices buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            vertex_num * 18,
        );

        let mesh_vertices_num_buffer = staged_buffer::StagedBuffer::<u32>::create_buffer(
            context.render_device,
            context.render_queue,
            &format!("terrain chunk vertices num buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            0,
        );

        let mesh_indices_num_buffer = staged_buffer::StagedBuffer::<u32>::create_buffer(
            context.render_device,
            context.render_queue,
            &format!("terrain chunk indices num buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            0,
        );

        Self {
            terrain_chunk_info_buffer,
            voxel_vertex_values_buffer,
            voxel_cross_points_buffer,
            mesh_vertex_map_buffer,
            mesh_vertex_location_buffer,
            mesh_vertex_normal_buffer,
            mesh_vertex_materials_buffer,
            mesh_indices_buffer,
            mesh_vertices_num_buffer,
            mesh_indices_num_buffer,
        }
    }

    pub fn stage_buffers(&self, command_encoder: &mut CommandEncoder) {
        self.mesh_vertex_location_buffer
            .stage_buffer(command_encoder);
        self.mesh_vertex_normal_buffer.stage_buffer(command_encoder);
        self.mesh_vertex_materials_buffer
            .stage_buffer(command_encoder);
        self.mesh_indices_buffer.stage_buffer(command_encoder);
        self.mesh_vertices_num_buffer.stage_buffer(command_encoder);
        self.mesh_indices_num_buffer.stage_buffer(command_encoder);
    }

    pub fn unmap(&self) {
        self.mesh_vertex_location_buffer.unmap();
        self.mesh_vertex_normal_buffer.unmap();
        self.mesh_vertex_materials_buffer.unmap();
        self.mesh_indices_buffer.unmap();
        self.mesh_vertices_num_buffer.unmap();
        self.mesh_indices_num_buffer.unmap();
    }
}

#[derive(Resource, Default)]
pub struct TerrainChunkMainBuffersCache {
    terrain_chunk_buffers: Vec<TerrainChunkMainBuffers>,
    used_count: usize,
}

impl TerrainChunkMainBuffersCache {
    pub fn acquire_terrain_chunk_buffers(&mut self) -> Option<usize> {
        if self.used_count < self.terrain_chunk_buffers.len() {
            self.used_count += 1;
            return Some(self.used_count - 1);
        }
        None
    }

    pub fn insert_terrain_chunk_buffers(&mut self, buffers: TerrainChunkMainBuffers) {
        self.terrain_chunk_buffers.push(buffers);
    }

    pub fn get_buffers_mut(
        &mut self,
        cached_id: TerrainChunkMainBufferCachedId,
    ) -> Option<&mut TerrainChunkMainBuffers> {
        self.terrain_chunk_buffers.get_mut(cached_id.0)
    }

    pub fn get_buffers(
        &self,
        cached_id: TerrainChunkMainBufferCachedId,
    ) -> Option<&TerrainChunkMainBuffers> {
        self.terrain_chunk_buffers.get(cached_id.0)
    }

    pub fn reset_used_count(&mut self) {
        self.used_count = 0;
    }
}

#[derive(Component, Deref, Copy, Clone)]
pub struct TerrainChunkMainBufferCachedId(pub usize);

pub struct TerrainChunkSeamBuffers {
    pub terrain_chunk_info_buffer: UniformBuffer<TerrainChunkInfo>,
    pub terrain_chunks_lod_buffer: UniformBuffer<[UVec4; 16]>,

    pub seam_mesh_vertex_map_buffer: BufferVec<u32>,
    pub seam_mesh_vertex_location_buffer: staged_buffer::StagedBufferVec<Vec4>,
    pub seam_mesh_vertex_normal_buffer: staged_buffer::StagedBufferVec<Vec4>,
    pub seam_mesh_vertex_materials_buffer: staged_buffer::StagedBufferVec<u32>,
    pub seam_mesh_indices_buffer: staged_buffer::StagedBufferVec<u32>,

    pub seam_mesh_vertices_num_buffer: staged_buffer::StagedBuffer<u32>,
    pub seam_mesh_indices_num_buffer: staged_buffer::StagedBuffer<u32>,
}

pub struct TerrainChunkSeamBufferCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub render_queue: &'a RenderQueue,
    pub terrain_chunk_aabb: Aabb3d,
    pub terrain_chunk_address: TerrainChunkAddress,
    pub terrain_chunk_seam_lod: TerrainChunkSeamLod,
    pub terrain_setting: &'a TerrainSetting,
}

impl TerrainChunkSeamBuffers {
    pub fn update_buffers_reuse_info(&mut self, context: TerrainChunkSeamBufferCreateContext) {
        // let max = context.terrain_chunk_seam_lod.get_max_lod();
        let add_lod = context.terrain_chunk_seam_lod.get_lod(SubNodeIndex::X0Y0Z0);
        let level = context.terrain_chunk_address.0.level() + add_lod[0];
        let voxel_size = context.terrain_setting.get_voxel_size(level);
        let chunk_size = context.terrain_setting.get_chunk_size(level - add_lod[0]);
        // let voxel_num = context.terrain_setting.get_voxel_num_in_chunk() * 2usize.pow(max as u32);
        let voxel_num = (chunk_size / voxel_size).round();

        let chunk_min = context.terrain_chunk_aabb.min;

        self.terrain_chunk_info_buffer.set(TerrainChunkInfo {
            chunk_min_location_size: Vec4::new(chunk_min.x, chunk_min.y, chunk_min.z, chunk_size),
            voxel_size,
            voxel_num: voxel_num as u32,
            qef_threshold: context.terrain_setting.qef_solver_threshold,
            qef_stddev: context.terrain_setting.qef_stddev,
        });
        self.terrain_chunk_info_buffer
            .write_buffer(context.render_device, context.render_queue);

        let lod = context.terrain_chunk_seam_lod.to_uniform_buffer_array();
        self.terrain_chunks_lod_buffer.set(lod);
        self.terrain_chunks_lod_buffer
            .write_buffer(context.render_device, context.render_queue);

        debug!(
            "TerrainChunkInfo: address: {:?}, voxel size: {}, voxel_num: {}, add_lod: {:?}",
            context.terrain_chunk_address, voxel_size, voxel_num, add_lod
        );

        self.seam_mesh_vertices_num_buffer.set_value(0);
        self.seam_mesh_vertices_num_buffer
            .write_buffer(context.render_device, context.render_queue);

        self.seam_mesh_indices_num_buffer.set_value(0);
        self.seam_mesh_indices_num_buffer
            .write_buffer(context.render_device, context.render_queue);
    }

    pub fn create_buffers(context: TerrainChunkSeamBufferCreateContext) -> TerrainChunkSeamBuffers {
        let chunk_min = context.terrain_chunk_aabb.min;
        let add_lod = context.terrain_chunk_seam_lod.get_lod(SubNodeIndex::X0Y0Z0);
        let level = context.terrain_chunk_address.0.level() + add_lod[0];
        let voxel_size = context.terrain_setting.get_voxel_size(level);
        let chunk_size = context.terrain_setting.get_chunk_size(level - add_lod[0]);

        // let max = context.terrain_chunk_seam_lod.get_max_lod();
        // context.terrain_setting.get_voxel_num_in_chunk() * 2usize.pow(max as u32);
        let voxel_num = (chunk_size / voxel_size).round() as usize;
        let total_voxel_num = (voxel_num + 1) * (voxel_num + 1) * 2;

        let terrain_chunk_info_buffer = {
            let mut chunk_info_uniform = UniformBuffer::from(TerrainChunkInfo {
                chunk_min_location_size: Vec4::new(
                    chunk_min.x,
                    chunk_min.y,
                    chunk_min.z,
                    chunk_size,
                ),
                voxel_size,
                voxel_num: voxel_num as u32,
                qef_threshold: context.terrain_setting.qef_solver_threshold,
                qef_stddev: context.terrain_setting.qef_stddev,
            });
            debug!(
                "TerrainChunkInfo: address: {:?}, voxel size: {}, voxel_num: {}, add_lod: {:?}",
                context.terrain_chunk_address, voxel_size, voxel_num, add_lod
            );
            chunk_info_uniform.set_label(Some("terrain chunk seam info uniform buffer"));
            chunk_info_uniform.write_buffer(context.render_device, context.render_queue);
            chunk_info_uniform
        };

        let terrain_chunks_lod_buffer = {
            let lod = context.terrain_chunk_seam_lod.to_uniform_buffer_array();
            let mut chunk_info_uniform = UniformBuffer::from(lod);
            chunk_info_uniform.set_label(Some("terrain chunk lod uniform buffer"));
            chunk_info_uniform.write_buffer(context.render_device, context.render_queue);
            chunk_info_uniform
        };

        let seam_mesh_vertex_map_buffer = {
            let mut vertex_buffer = BufferVec::<u32>::new(BufferUsages::STORAGE);
            vertex_buffer.set_label(Some("terrain chunk mesh vertex map buffer"));
            vertex_buffer.reserve(total_voxel_num, context.render_device);
            vertex_buffer
        };

        let seam_mesh_vertex_location_buffer =
            staged_buffer::StagedBufferVec::<Vec4>::create_buffer(
                context.render_device,
                &format!("terrain chunk vertex location buffer {:?}", chunk_min),
                BufferUsages::STORAGE,
                total_voxel_num,
            );

        let seam_mesh_vertex_normal_buffer = staged_buffer::StagedBufferVec::<Vec4>::create_buffer(
            context.render_device,
            &format!("terrain chunk vertex normal buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            total_voxel_num,
        );

        let seam_mesh_vertex_materials_buffer =
            staged_buffer::StagedBufferVec::<u32>::create_buffer(
                context.render_device,
                &format!("terrain chunk vertex material buffer {:?}", chunk_min),
                BufferUsages::STORAGE,
                total_voxel_num,
            );

        let seam_mesh_indices_buffer = staged_buffer::StagedBufferVec::<u32>::create_buffer(
            context.render_device,
            &format!("terrain chunk indices buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            total_voxel_num * 18,
        );

        let seam_mesh_vertices_num_buffer = staged_buffer::StagedBuffer::<u32>::create_buffer(
            context.render_device,
            context.render_queue,
            &format!("terrain chunk vertices num buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            0,
        );

        let seam_mesh_indices_num_buffer = staged_buffer::StagedBuffer::<u32>::create_buffer(
            context.render_device,
            context.render_queue,
            &format!("terrain chunk indices num buffer {:?}", chunk_min),
            BufferUsages::STORAGE,
            0,
        );

        Self {
            terrain_chunk_info_buffer,
            terrain_chunks_lod_buffer,
            seam_mesh_vertex_map_buffer,
            seam_mesh_vertex_location_buffer,
            seam_mesh_vertex_normal_buffer,
            seam_mesh_vertex_materials_buffer,
            seam_mesh_indices_buffer,
            seam_mesh_vertices_num_buffer,
            seam_mesh_indices_num_buffer,
        }
    }

    pub fn stage_buffers(&self, command_encoder: &mut CommandEncoder) {
        self.seam_mesh_vertex_location_buffer
            .stage_buffer(command_encoder);
        self.seam_mesh_vertex_normal_buffer
            .stage_buffer(command_encoder);
        self.seam_mesh_vertex_materials_buffer
            .stage_buffer(command_encoder);
        self.seam_mesh_indices_buffer.stage_buffer(command_encoder);
        self.seam_mesh_vertices_num_buffer
            .stage_buffer(command_encoder);
        self.seam_mesh_indices_num_buffer
            .stage_buffer(command_encoder);
    }

    pub fn unmap(&self) {
        self.seam_mesh_vertex_location_buffer.unmap();
        self.seam_mesh_vertex_normal_buffer.unmap();
        self.seam_mesh_vertex_materials_buffer.unmap();
        self.seam_mesh_indices_buffer.unmap();
        self.seam_mesh_vertices_num_buffer.unmap();
        self.seam_mesh_indices_num_buffer.unmap();
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct TerrainChunkSeamKey {
    pub lod: TerrainChunkSeamLod,
}

pub struct TerrainChunkSeamBuffersCounter {
    terrain_chunk_seam_buffers: Vec<TerrainChunkSeamBuffers>,
    used_count: usize,
}

#[derive(Resource, Default)]
pub struct TerrainChunkSeamBuffersCache {
    terrain_chunk_seam_buffers_map: HashMap<TerrainChunkSeamKey, TerrainChunkSeamBuffersCounter>,
}

impl TerrainChunkSeamBuffersCache {
    pub fn acquire_terrain_chunk_buffers(&mut self, key: TerrainChunkSeamKey) -> Option<usize> {
        if let Some(buffers_counter) = self.terrain_chunk_seam_buffers_map.get_mut(&key) {
            if buffers_counter.used_count < buffers_counter.terrain_chunk_seam_buffers.len() {
                buffers_counter.used_count += 1;
                return Some(buffers_counter.used_count - 1);
            }
        }
        None
    }

    pub fn insert_terrain_chunk_buffers(
        &mut self,
        key: TerrainChunkSeamKey,
        buffers: TerrainChunkSeamBuffers,
    ) {
        let buffers_counter = self
            .terrain_chunk_seam_buffers_map
            .entry(key)
            .or_insert_with(|| TerrainChunkSeamBuffersCounter {
                terrain_chunk_seam_buffers: Vec::new(),
                used_count: 0,
            });
        buffers_counter.terrain_chunk_seam_buffers.push(buffers);
    }

    pub fn get_buffers_mut(
        &mut self,
        key: TerrainChunkSeamKey,
        cached_id: usize,
    ) -> Option<&mut TerrainChunkSeamBuffers> {
        if let Some(buffers_counter) = self.terrain_chunk_seam_buffers_map.get_mut(&key) {
            buffers_counter
                .terrain_chunk_seam_buffers
                .get_mut(cached_id)
        } else {
            None
        }
    }

    pub fn get_buffers(
        &self,
        key: TerrainChunkSeamKey,
        cached_id: usize,
    ) -> Option<&TerrainChunkSeamBuffers> {
        if let Some(buffers_counter) = self.terrain_chunk_seam_buffers_map.get(&key) {
            buffers_counter.terrain_chunk_seam_buffers.get(cached_id)
        } else {
            None
        }
    }

    pub fn reset_used_count(&mut self) {
        for value in self.terrain_chunk_seam_buffers_map.values_mut() {
            value.used_count = 0;
        }
    }
}

#[derive(Component, Deref, Copy, Clone)]
pub struct TerrainChunkSeamBufferCachedId(pub [usize; 3]);
