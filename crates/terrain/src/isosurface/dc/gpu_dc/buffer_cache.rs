use bevy::{
    math::bounding::Aabb3d,
    prelude::*,
    render::{
        render_resource::{CommandEncoder, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
    utils::HashMap,
};
use bitflags::bitflags;
use wgpu::{BufferAddress, BufferSize, DynamicOffset};

use crate::{
    chunk_mgr::chunk::comp::TerrainChunkAddress,
    isosurface::dc::gpu_dc::buffer_type::TerrainChunkVerticesIndicesCount, setting::TerrainSetting,
};

use super::{
    buffer_type::{
        TerrainChunkCSGOperation, TerrainChunkInfo, TerrainChunkMeshIndicesVec,
        TerrainChunkMeshVertexInfoVec, TerrainChunkMeshVertexMapVec, TerrainChunkVertexInfo,
        VoxelEdgeCrossPoint, VoxelEdgeCrossPointVec, VoxelVertexValueVec,
    },
    shared_buffer::{SharedStorageBuffer, SharedUniformBuffer},
    staged_buffer,
};

bitflags! {
    #[derive(Debug, PartialEq, Eq, Default)]
    pub struct TerrainChunkMainRecreateBindGroup: u8 {
        const None = 0;
        const MainMesh = 1;
        const CSG = 2;
    }
}

#[derive(Resource)]
pub struct TerrainChunkMainDynamicBuffers {
    pub terrain_chunk_info_dynamic_buffer: SharedUniformBuffer<TerrainChunkInfo>,
    pub voxel_vertex_values_dynamic_buffer: SharedStorageBuffer<VoxelVertexValueVec>,
    pub voxel_cross_points_dynamic_buffer: SharedStorageBuffer<VoxelEdgeCrossPointVec>,

    pub mesh_vertex_map_dynamic_buffer: SharedStorageBuffer<TerrainChunkMeshVertexMapVec>,

    pub mesh_vertices_dynamic_buffer:
        staged_buffer::SharedStagedBuffer<TerrainChunkMeshVertexInfoVec>,
    pub mesh_indices_dynamic_buffer: staged_buffer::SharedStagedBuffer<TerrainChunkMeshIndicesVec>,

    pub mesh_vertices_indices_count_dynamic_buffer:
        staged_buffer::SharedStagedBuffer<TerrainChunkVerticesIndicesCount>,

    pub csg_info_dynamic_buffer: SharedUniformBuffer<u32>,
    pub csg_operations_dynamic_buffer: SharedStorageBuffer<Vec<TerrainChunkCSGOperation>>,

    pub terrain_chunk_buffer_bindings_map: HashMap<Entity, TerrainChunkMainBufferBindings>,

    pub recreate_bind_group: TerrainChunkMainRecreateBindGroup,
}

pub struct TerrainChunkMainDynamicBufferReserveContext<'a> {
    pub render_device: &'a RenderDevice,
    pub render_queue: &'a RenderQueue,
    pub terrain_setting: &'a TerrainSetting,
    pub instance_num: usize,
    pub csg_operations_num: usize,
}

impl TerrainChunkMainDynamicBuffers {
    pub fn insert_terrain_chunk_buffer_bindings(
        &mut self,
        entity: Entity,
        buffer_bindings: TerrainChunkMainBufferBindings,
    ) {
        self.terrain_chunk_buffer_bindings_map
            .insert(entity, buffer_bindings);
    }

    pub fn get_buffers_dynamic_offset(&self, entity: Entity, group_id: u32) -> Vec<DynamicOffset> {
        if let Some(bindings) = self.terrain_chunk_buffer_bindings_map.get(&entity) {
            return bindings.get_dynamic_offset(group_id);
        }
        vec![]
    }

    pub fn get_csg_operations_binding_info(&self, entity: Entity) -> DynamicBufferBindingInfo {
        let bindings = self.terrain_chunk_buffer_bindings_map.get(&entity).unwrap();
        bindings.csg_operations_buffer_binding
    }

    pub fn get_buffer_bindings(&self, entity: Entity) -> Option<&TerrainChunkMainBufferBindings> {
        self.terrain_chunk_buffer_bindings_map.get(&entity)
    }

    pub fn clear(&mut self) {
        let _span = info_span!("terrain chunk main dynamic buffers clear").entered();

        self.terrain_chunk_buffer_bindings_map.clear();

        self.terrain_chunk_info_dynamic_buffer.clear();
        self.voxel_vertex_values_dynamic_buffer.clear();
        self.voxel_cross_points_dynamic_buffer.clear();

        self.mesh_vertex_map_dynamic_buffer.clear();

        self.mesh_vertices_dynamic_buffer.clear();
        self.mesh_indices_dynamic_buffer.clear();

        self.mesh_vertices_indices_count_dynamic_buffer.clear();

        self.csg_info_dynamic_buffer.clear();
        self.csg_operations_dynamic_buffer.clear();

        self.recreate_bind_group = TerrainChunkMainRecreateBindGroup::None;
    }
}

impl TerrainChunkMainDynamicBuffers {
    pub fn new(render_device: &RenderDevice) -> TerrainChunkMainDynamicBuffers {
        let storage_buffer_alignment =
            render_device.limits().min_storage_buffer_offset_alignment as u64;
        let uniform_buffer_alignment =
            render_device.limits().min_uniform_buffer_offset_alignment as u64;

        let mut terrain_chunk_info_dynamic_buffer =
            SharedUniformBuffer::<TerrainChunkInfo>::new_with_alignment(uniform_buffer_alignment);
        terrain_chunk_info_dynamic_buffer.set_label(Some("terrain chunk info uniform buffer"));

        let mut voxel_vertex_values_dynamic_buffer =
            SharedStorageBuffer::<VoxelVertexValueVec>::new(storage_buffer_alignment);
        voxel_vertex_values_dynamic_buffer
            .set_label(Some("terrain chunk voxel vertex values buffer"));

        let mut voxel_cross_points_dynamic_buffer =
            SharedStorageBuffer::<VoxelEdgeCrossPointVec>::new(storage_buffer_alignment);
        voxel_cross_points_dynamic_buffer.set_label(Some("terrain chunk voxel cross point buffer"));

        let mut mesh_vertex_map_dynamic_buffer =
            SharedStorageBuffer::<TerrainChunkMeshVertexMapVec>::new(storage_buffer_alignment);
        mesh_vertex_map_dynamic_buffer.set_label(Some("terrain chunk mesh vertex map buffer"));

        let mut mesh_vertices_dynamic_buffer = staged_buffer::SharedStagedBuffer::<
            TerrainChunkMeshVertexInfoVec,
        >::new(storage_buffer_alignment);
        mesh_vertices_dynamic_buffer.set_label("terrain chunk vertices buffer");

        let mut mesh_indices_dynamic_buffer = staged_buffer::SharedStagedBuffer::<
            TerrainChunkMeshIndicesVec,
        >::new(storage_buffer_alignment);
        mesh_indices_dynamic_buffer.set_label("terrain chunk indices buffer");

        let mut mesh_vertices_indices_count_dynamic_buffer =
            staged_buffer::SharedStagedBuffer::<TerrainChunkVerticesIndicesCount>::new(
                storage_buffer_alignment,
            );
        mesh_vertices_indices_count_dynamic_buffer.set_label("terrain chunk vertices num buffer");

        let mut csg_info_dynamic_buffer =
            SharedUniformBuffer::<u32>::new_with_alignment(uniform_buffer_alignment);
        csg_info_dynamic_buffer.set_label(Some("terrain chunk mesh csg info buffer"));

        let mut csg_operations_dynamic_buffer =
            SharedStorageBuffer::<Vec<TerrainChunkCSGOperation>>::new(storage_buffer_alignment);
        csg_operations_dynamic_buffer.set_label(Some("terrain chunk mesh csg operations buffer"));

        Self {
            terrain_chunk_info_dynamic_buffer,
            voxel_vertex_values_dynamic_buffer,
            voxel_cross_points_dynamic_buffer,
            mesh_vertex_map_dynamic_buffer,
            mesh_vertices_dynamic_buffer,
            mesh_indices_dynamic_buffer,
            mesh_vertices_indices_count_dynamic_buffer,
            csg_info_dynamic_buffer,
            csg_operations_dynamic_buffer,

            terrain_chunk_buffer_bindings_map: HashMap::new(),
            recreate_bind_group: TerrainChunkMainRecreateBindGroup::None,
        }
    }

    pub fn set_stride(&mut self, terrain_setting: &TerrainSetting) {
        let voxel_num = terrain_setting.get_voxel_num_in_chunk();
        let voxel_vertex_num = voxel_num + 1;

        let total_voxel_num = (voxel_num * voxel_num * voxel_num) as u64;
        let mesh_vertex_num = (voxel_num * voxel_num * 10) as u64;
        let total_voxel_vertex_num =
            (voxel_vertex_num * voxel_vertex_num * voxel_vertex_num) as u64;

        self.voxel_vertex_values_dynamic_buffer
            .set_stride(BufferSize::new(f32::min_size().get() * total_voxel_vertex_num).unwrap());
        self.voxel_cross_points_dynamic_buffer.set_stride(
            BufferSize::new(VoxelEdgeCrossPoint::min_size().get() * total_voxel_vertex_num * 3)
                .unwrap(),
        );
        self.mesh_vertex_map_dynamic_buffer
            .set_stride(BufferSize::new(u32::min_size().get() * total_voxel_num).unwrap());
        self.mesh_vertices_dynamic_buffer.set_stride(
            BufferSize::new(TerrainChunkVertexInfo::min_size().get() * mesh_vertex_num).unwrap(),
        );
        self.mesh_indices_dynamic_buffer
            .set_stride(BufferSize::new(u32::min_size().get() * mesh_vertex_num * 18).unwrap());
    }

    pub fn reserve_buffers(&mut self, context: &TerrainChunkMainDynamicBufferReserveContext) {
        let _span = info_span!("terrain chunk main buffers reverse").entered();

        {
            let _span = info_span!("terrain chunk main info buffers reserve").entered();
            if self
                .terrain_chunk_info_dynamic_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        }

        {
            let _span =
                info_span!("terrain chunk main voxel vertex values buffers reserve").entered();
            if self
                .voxel_vertex_values_dynamic_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        };

        {
            let _span =
                info_span!("terrain chunk main voxel cross point buffers reserve").entered();
            if self
                .voxel_cross_points_dynamic_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        };

        {
            let _span = info_span!("terrain chunk main mesh vertex map buffers reserve").entered();
            if self
                .mesh_vertex_map_dynamic_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        };

        {
            let _span = info_span!("terrain chunk main mesh vertex stage buffers create").entered();
            if self
                .mesh_vertices_dynamic_buffer
                .reserve_buffer(context.render_device, context.instance_num)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        };

        {
            let _span =
                info_span!("terrain chunk main mesh indices stage buffers create").entered();
            if self
                .mesh_indices_dynamic_buffer
                .reserve_buffer(context.render_device, context.instance_num)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        };

        {
            let _span =
                info_span!("terrain chunk main mesh vertices indices count stage buffers reserve")
                    .entered();
            if self
                .mesh_vertices_indices_count_dynamic_buffer
                .reserve_buffer(context.render_device, context.instance_num)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
            }
        };

        {
            let _span = info_span!("terrain chunk main csg info buffers reserve").entered();
            if self
                .csg_info_dynamic_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::CSG;
            }
        }

        // {
        //     let _span = info_span!("terrain chunk main csg operations buffers reserve").entered();
        //     if self
        //         .csg_operations_dynamic_buffer
        //         .reserve_buffer(context.csg_operations_num, context.render_device)
        //     {
        //         self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::CSG;
        //     }
        // };
    }

    pub fn set_buffers_data(&mut self, context: TerrainChunkMainBufferCreateContext) {
        let _span = info_span!("terrain chunk main buffers write buffers data").entered();

        let level = context.terrain_chunk_address.0.depth();
        let chunk_size = context.terrain_setting.get_chunk_size(level);
        let voxel_size = context.terrain_setting.get_voxel_size(level);
        let voxel_num = context.terrain_setting.get_voxel_num_in_chunk();

        let chunk_min = context.terrain_chunk_aabb.min;

        {
            let _span = info_span!("terrain chunk main chunk info buffers write buffer").entered();

            self.terrain_chunk_info_dynamic_buffer
                .push(&TerrainChunkInfo {
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
        }

        {
            let _span =
                info_span!("terrain chunk main vertices indices count buffers write buffer")
                    .entered();
            self.mesh_vertices_indices_count_dynamic_buffer.push_value(
                TerrainChunkVerticesIndicesCount {
                    vertices_count: 0,
                    indices_count: 0,
                },
            );
        }

        {
            let _span =
                info_span!("terrain chunk main vertices indices count buffers write buffer")
                    .entered();
            if let Some(csg_operators) = context.terrain_chunk_csg_operations {
                self.csg_info_dynamic_buffer
                    .push(&(csg_operators.len() as u32));
            } else {
                self.csg_info_dynamic_buffer.push(&0);
            }
        }

        {
            let _span =
                info_span!("terrain chunk main csg operations buffers write buffer").entered();

            let offset;
            let size;
            if let Some(csg_operations) = context.terrain_chunk_csg_operations {
                offset = self
                    .csg_operations_dynamic_buffer
                    .push(csg_operations.clone());
                size = csg_operations.len() as u64 * TerrainChunkCSGOperation::min_size().get();
            } else {
                offset = self
                    .csg_operations_dynamic_buffer
                    .push(vec![TerrainChunkCSGOperation {
                        location: Vec3::ZERO,
                        primitive_type: 100,
                        shape: Vec3::ZERO,
                        operation_type: 10000,
                    }]);
                size = TerrainChunkCSGOperation::min_size().get();
            }

            let binding = self
                .terrain_chunk_buffer_bindings_map
                .get_mut(&context.entity)
                .unwrap();
            binding.csg_operations_buffer_binding = DynamicBufferBindingInfo::new(
                offset as u64,
                BufferSize::new(
                    self.csg_operations_dynamic_buffer
                        .get_alignment_value()
                        .round_up(size),
                ),
            );
        }
    }

    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        if self
            .terrain_chunk_info_dynamic_buffer
            .write_buffer(render_device, render_queue)
        {
            self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::MainMesh;
        }

        self.mesh_vertices_indices_count_dynamic_buffer
            .write_buffer(render_device, render_queue);

        if self
            .csg_info_dynamic_buffer
            .write_buffer(render_device, render_queue)
        {
            self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::CSG;
        }

        if self
            .csg_operations_dynamic_buffer
            .reserve_buffer_to_scratch(render_device)
        {
            self.recreate_bind_group |= TerrainChunkMainRecreateBindGroup::CSG;
        }

        self.csg_operations_dynamic_buffer
            .write_buffer(render_device, render_queue)
    }

    pub fn stage_buffers(&self, command_encoder: &mut CommandEncoder) {
        self.mesh_vertices_dynamic_buffer.stage_buffer(
            command_encoder,
            self.mesh_vertices_dynamic_buffer
                .cpu_buffer
                .as_ref()
                .unwrap()
                .size(),
        );
        self.mesh_indices_dynamic_buffer.stage_buffer(
            command_encoder,
            self.mesh_indices_dynamic_buffer
                .cpu_buffer
                .as_ref()
                .unwrap()
                .size(),
        );
        self.mesh_vertices_indices_count_dynamic_buffer
            .stage_buffer(
                command_encoder,
                self.mesh_vertices_indices_count_dynamic_buffer
                    .cpu_buffer
                    .as_ref()
                    .unwrap()
                    .size(),
            );
    }

    pub fn unmap(&self) {
        self.mesh_vertices_dynamic_buffer.unmap();
        self.mesh_indices_dynamic_buffer.unmap();
        self.mesh_vertices_indices_count_dynamic_buffer.unmap();
    }

    pub fn map_async(&self) {
        self.mesh_vertices_dynamic_buffer.map_async(..);
        self.mesh_indices_dynamic_buffer.map_async(..);
        self.mesh_vertices_indices_count_dynamic_buffer
            .map_async(..);
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct DynamicBufferBindingInfo {
    pub offset: BufferAddress,
    pub size: Option<BufferSize>,
}

impl DynamicBufferBindingInfo {
    pub fn new(offset: BufferAddress, size: Option<BufferSize>) -> Self {
        Self { offset, size }
    }

    pub fn from_num(last_num: usize, current_num: usize, type_size: u64) -> Self {
        Self {
            offset: type_size * last_num as u64,
            size: Some(BufferSize::new(type_size * current_num as u64).unwrap()),
        }
    }

    pub fn get_right_offset(&self) -> BufferAddress {
        self.offset + self.size.unwrap().get()
    }
}

#[derive(Default, Debug)]
pub struct TerrainChunkMainBufferBindings {
    pub terrain_chunk_info_buffer_binding: DynamicBufferBindingInfo,
    pub voxel_vertex_values_buffer_binding: DynamicBufferBindingInfo,
    pub voxel_cross_points_buffer_binding: DynamicBufferBindingInfo,

    pub mesh_vertex_map_buffer_binding: DynamicBufferBindingInfo,

    pub mesh_vertices_buffer_binding: DynamicBufferBindingInfo,
    pub mesh_indices_buffer_binding: DynamicBufferBindingInfo,

    pub mesh_vertices_indices_count_buffer_binding: DynamicBufferBindingInfo,

    pub csg_info_buffer_binding: DynamicBufferBindingInfo,
    pub csg_operations_buffer_binding: DynamicBufferBindingInfo,
}

pub struct TerrainChunkMainBufferCreateContext<'a> {
    pub terrain_chunk_aabb: Aabb3d,
    pub terrain_chunk_address: TerrainChunkAddress,
    pub terrain_setting: &'a TerrainSetting,
    pub terrain_chunk_csg_operations: &'a Option<Vec<TerrainChunkCSGOperation>>,
    pub entity: Entity,
}

pub struct TerrainChunkMainBufferBindingsBuilder<'a> {
    pub current_index: usize,

    pub last_csg_operations_num: usize,
    pub current_csg_operations_num: usize,

    pub terrain_setting: &'a TerrainSetting,
    pub dynamic_buffers: &'a TerrainChunkMainDynamicBuffers,
}

impl TerrainChunkMainBufferBindings {
    pub fn get_dynamic_offset(&self, group_id: u32) -> Vec<DynamicOffset> {
        if group_id == 0 {
            vec![
                self.terrain_chunk_info_buffer_binding.offset as DynamicOffset,
                self.voxel_vertex_values_buffer_binding.offset as DynamicOffset,
                self.voxel_cross_points_buffer_binding.offset as DynamicOffset,
                self.mesh_vertices_buffer_binding.offset as DynamicOffset,
                self.mesh_indices_buffer_binding.offset as DynamicOffset,
                self.mesh_vertex_map_buffer_binding.offset as DynamicOffset,
                self.mesh_vertices_indices_count_buffer_binding.offset as DynamicOffset,
            ]
        } else {
            vec![
                self.csg_info_buffer_binding.offset as DynamicOffset,
                self.csg_operations_buffer_binding.offset as DynamicOffset,
            ]
        }
    }

    pub fn rebuild_binding_size(&mut self, builder: TerrainChunkMainBufferBindingsBuilder) {
        let last_index = builder.current_index - 1;

        self.terrain_chunk_info_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .terrain_chunk_info_dynamic_buffer
                .get_alignment(),
        );

        self.voxel_vertex_values_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .voxel_vertex_values_dynamic_buffer
                .get_stride_alignment(),
        );

        self.voxel_cross_points_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .voxel_cross_points_dynamic_buffer
                .get_stride_alignment(),
        );

        self.mesh_vertex_map_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .mesh_vertex_map_dynamic_buffer
                .get_stride_alignment(),
        );

        self.mesh_vertices_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .mesh_vertices_dynamic_buffer
                .get_alignment(),
        );

        self.mesh_indices_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .mesh_indices_dynamic_buffer
                .get_alignment(),
        );

        self.mesh_vertices_indices_count_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .mesh_vertices_indices_count_dynamic_buffer
                .get_alignment(),
        );

        self.csg_info_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .dynamic_buffers
                .csg_info_dynamic_buffer
                .get_alignment(),
        );
    }
}
