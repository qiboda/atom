use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::chunk_mgr::chunk::comp::TerrainChunkAabb;

use super::{
    buffer_cache::{TerrainChunkMainBuffers, TerrainChunkSeamBuffers, TerrainChunkSeamKey},
    pipelines::TerrainChunkPipelines,
};

pub struct TerrainChunkMainBindGroups {
    pub main_mesh_bind_group: BindGroup,
}

pub struct TerrainChunkMainBindGroupsCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub pipelines: &'a TerrainChunkPipelines,
    pub buffers: &'a TerrainChunkMainBuffers,
    pub aabb: &'a TerrainChunkAabb,
}

impl TerrainChunkMainBindGroups {
    pub fn create_bind_groups(context: TerrainChunkMainBindGroupsCreateContext) -> Self {
        let mesh_vertices_bind_group: BindGroup = context.render_device.create_bind_group(
            "terrain chunk mesh vertices bind group",
            &context.pipelines.main_compute_bind_group_layout,
            &BindGroupEntries::sequential((
                context.buffers.terrain_chunk_info_buffer.binding().unwrap(),
                context
                    .buffers
                    .voxel_vertex_values_buffer
                    .binding()
                    .unwrap(),
                context.buffers.voxel_cross_points_buffer.binding().unwrap(),
                context
                    .buffers
                    .mesh_vertices_buffer
                    .get_gpu_buffer()
                    .binding()
                    .unwrap(),
                context
                    .buffers
                    .mesh_indices_buffer
                    .get_gpu_buffer()
                    .binding()
                    .unwrap(),
                context.buffers.mesh_vertex_map_buffer.binding().unwrap(),
                context
                    .buffers
                    .mesh_vertices_indices_count_buffer
                    .get_gpu_buffer()
                    .binding()
                    .unwrap(),
            )),
        );

        Self {
            main_mesh_bind_group: mesh_vertices_bind_group,
        }
    }
}

#[derive(Resource, Default)]
pub struct TerrainChunkMainBindGroupsCache {
    pub terrain_chunk_bind_groups: Vec<TerrainChunkMainBindGroups>,
    pub used_count: usize,
}

impl TerrainChunkMainBindGroupsCache {
    pub fn acquire_terrain_chunk_bind_group(&mut self) -> Option<usize> {
        if self.used_count < self.terrain_chunk_bind_groups.len() {
            self.used_count += 1;
            return Some(self.used_count - 1);
        }
        None
    }

    pub fn insert(&mut self, bind_groups: TerrainChunkMainBindGroups) {
        self.terrain_chunk_bind_groups.push(bind_groups);
    }

    pub fn get_bind_groups(
        &self,
        id: TerrainChunkMainBindGroupCachedId,
    ) -> Option<&TerrainChunkMainBindGroups> {
        self.terrain_chunk_bind_groups.get(id.0)
    }

    pub fn reset_used_count(&mut self) {
        self.used_count = 0;
    }
}

#[derive(Component, Deref, Copy, Clone)]
pub struct TerrainChunkMainBindGroupCachedId(pub usize);

pub struct TerrainChunkSeamBindGroups {
    pub seam_mesh_bind_group: BindGroup,
}

pub struct TerrainChunkSeamBindGroupsCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub pipelines: &'a TerrainChunkPipelines,
    pub buffers: &'a TerrainChunkSeamBuffers,
    pub aabb: &'a TerrainChunkAabb,
}

impl TerrainChunkSeamBindGroups {
    pub fn create_bind_groups(context: TerrainChunkSeamBindGroupsCreateContext) -> Self {
        let seam_mesh_bind_group: BindGroup = context.render_device.create_bind_group(
            "terrain chunk seam mesh vertices bind group",
            &context.pipelines.seam_compute_bind_group_layout,
            &BindGroupEntries::sequential((
                context.buffers.terrain_chunk_info_buffer.binding().unwrap(),
                context.buffers.terrain_chunks_lod_buffer.binding().unwrap(),
                context
                    .buffers
                    .seam_mesh_vertices_buffer
                    .get_gpu_buffer()
                    .binding()
                    .unwrap(),
                context
                    .buffers
                    .seam_mesh_indices_buffer
                    .get_gpu_buffer()
                    .binding()
                    .unwrap(),
                context
                    .buffers
                    .seam_mesh_vertex_map_buffer
                    .binding()
                    .unwrap(),
                context
                    .buffers
                    .seam_mesh_vertices_indices_count_buffer
                    .get_gpu_buffer()
                    .binding()
                    .unwrap(),
            )),
        );

        Self {
            seam_mesh_bind_group,
        }
    }
}

pub struct TerrainChunkSeamBindGroupsCounter {
    terrain_chunk_seam_bind_groups: Vec<TerrainChunkSeamBindGroups>,
    used_count: usize,
}

#[derive(Resource, Default)]
pub struct TerrainChunkSeamBindGroupsCache {
    pub terrain_chunk_bind_groups_map:
        HashMap<TerrainChunkSeamKey, TerrainChunkSeamBindGroupsCounter>,
}

impl TerrainChunkSeamBindGroupsCache {
    pub fn acquire_terrain_chunk_bind_group(&mut self, key: TerrainChunkSeamKey) -> Option<usize> {
        if let Some(bind_groups_counter) = self.terrain_chunk_bind_groups_map.get_mut(&key) {
            if bind_groups_counter.used_count
                < bind_groups_counter.terrain_chunk_seam_bind_groups.len()
            {
                bind_groups_counter.used_count += 1;
                return Some(bind_groups_counter.used_count - 1);
            }
        }
        None
    }

    pub fn insert(&mut self, key: TerrainChunkSeamKey, bind_groups: TerrainChunkSeamBindGroups) {
        let bind_groups_counter = self
            .terrain_chunk_bind_groups_map
            .entry(key)
            .or_insert_with(|| TerrainChunkSeamBindGroupsCounter {
                terrain_chunk_seam_bind_groups: Vec::new(),
                used_count: 0,
            });
        bind_groups_counter
            .terrain_chunk_seam_bind_groups
            .push(bind_groups);
    }

    pub fn get_bind_groups(
        &self,
        key: TerrainChunkSeamKey,
        id: usize,
    ) -> Option<&TerrainChunkSeamBindGroups> {
        if let Some(bind_groups_counter) = self.terrain_chunk_bind_groups_map.get(&key) {
            bind_groups_counter.terrain_chunk_seam_bind_groups.get(id)
        } else {
            None
        }
    }

    pub fn reset_used_count(&mut self) {
        for value in self.terrain_chunk_bind_groups_map.values_mut() {
            value.used_count = 0;
        }
    }
}

#[derive(Component, Deref, Copy, Clone)]
pub struct TerrainChunkSeamBindGroupCachedId(pub [usize; 3]);
