use atom_render::{
    shared_buffer::{SharedStorageBuffer, SharedUniformBuffer},
    staged_buffer::SharedStagedBuffer,
};
use bevy::{
    platform::collections::HashMap,
    prelude::*,
    render::{
        render_resource::{BufferAddress, BufferSize, CommandEncoder, DynamicOffset, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::{
    chunks::{
        chunk::TerrainChunkCoord,
        mesh::{
            components::TerrainChunkMeshingState, compute::types::TerrainChunkVerticesIndicesCount,
        },
    },
    terrain::setting::TerrainSetting,
};

use super::types::{
    TerrainChunkInfo, TerrainChunkMeshIndicesVec, TerrainChunkMeshVertexInfoVec,
    TerrainChunkMeshVertexMapVec, TerrainChunkVertexInfo, VoxelEdgeCrossPoint,
    VoxelEdgeCrossPointVec, VoxelVertexValueVec,
};

#[derive(Resource)]
pub struct TerrainChunkMeshBuffers {
    pub terrain_chunk_info_buffer: SharedUniformBuffer<TerrainChunkInfo>,
    /**
     * 用于跨compute shader传递体素顶点的 isosurface 值
     */
    pub voxel_vertex_values_buffer: SharedStorageBuffer<VoxelVertexValueVec>,
    /**
     * 用于跨compute shader传递体素的边的相交点信息
     */
    pub voxel_cross_points_buffer: SharedStorageBuffer<VoxelEdgeCrossPointVec>,

    /**
     * 用于根据体素的索引，快速找到对应的顶点索引
     */
    pub mesh_vertex_map_buffer: SharedStorageBuffer<TerrainChunkMeshVertexMapVec>,

    /**
     * 存储了compute shader计算得出的地形块的顶点信息
     */
    pub mesh_vertices_buffer: SharedStagedBuffer<TerrainChunkMeshVertexInfoVec>,
    /**
     * 存储了compute shader计算得出的地形块的索引信息
     */
    pub mesh_indices_buffer: SharedStagedBuffer<TerrainChunkMeshIndicesVec>,

    /**
     * 存储了每个地形块的顶点和索引数量
     */
    pub mesh_vertices_indices_count_buffer: SharedStagedBuffer<TerrainChunkVerticesIndicesCount>,

    /**
     * 每个 entity 对应的 buffer 绑定信息
     */
    pub terrain_chunk_buffer_bindings_map: HashMap<Entity, TerrainChunkMeshBufferBindings>,

    /**
     * 是否应该创建 BindGroup
     */
    pub should_create: bool,
}

pub struct TerrainChunkMeshBufferReserveContext<'a> {
    pub render_device: &'a RenderDevice,
    pub render_queue: &'a RenderQueue,
    pub terrain_setting: &'a TerrainSetting,
    pub instance_num: usize,
}

impl TerrainChunkMeshBuffers {
    pub fn insert_terrain_chunk_buffer_bindings(
        &mut self,
        entity: Entity,
        buffer_bindings: TerrainChunkMeshBufferBindings,
    ) {
        self.terrain_chunk_buffer_bindings_map
            .insert(entity, buffer_bindings);
    }

    pub fn get_buffers_dynamic_offset(&self, entity: Entity) -> Vec<DynamicOffset> {
        if let Some(bindings) = self.terrain_chunk_buffer_bindings_map.get(&entity) {
            return bindings.get_dynamic_offset();
        }
        vec![]
    }

    pub fn get_buffer_bindings(&self, entity: Entity) -> Option<&TerrainChunkMeshBufferBindings> {
        self.terrain_chunk_buffer_bindings_map.get(&entity)
    }

    pub fn clear(&mut self) {
        let _span = info_span!("terrain chunk mesh buffers clear").entered();

        self.terrain_chunk_buffer_bindings_map.clear();

        self.terrain_chunk_info_buffer.clear();
        self.voxel_vertex_values_buffer.clear();
        self.voxel_cross_points_buffer.clear();

        self.mesh_vertex_map_buffer.clear();

        self.mesh_vertices_buffer.clear();
        self.mesh_indices_buffer.clear();

        self.mesh_vertices_indices_count_buffer.clear();

        self.should_create = false;
    }
}

impl TerrainChunkMeshBuffers {
    pub fn new(render_device: &RenderDevice) -> TerrainChunkMeshBuffers {
        // 因为 buffer 比较大，因此尽可能使用小的对齐方式，以节省内存
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

        let mut mesh_vertices_dynamic_buffer =
            SharedStagedBuffer::<TerrainChunkMeshVertexInfoVec>::new(storage_buffer_alignment);
        mesh_vertices_dynamic_buffer.set_label("terrain chunk vertices buffer");

        let mut mesh_indices_dynamic_buffer =
            SharedStagedBuffer::<TerrainChunkMeshIndicesVec>::new(storage_buffer_alignment);
        mesh_indices_dynamic_buffer.set_label("terrain chunk indices buffer");

        let mut mesh_vertices_indices_count_dynamic_buffer =
            SharedStagedBuffer::<TerrainChunkVerticesIndicesCount>::new(storage_buffer_alignment);
        mesh_vertices_indices_count_dynamic_buffer.set_label("terrain chunk vertices num buffer");

        Self {
            terrain_chunk_info_buffer: terrain_chunk_info_dynamic_buffer,
            voxel_vertex_values_buffer: voxel_vertex_values_dynamic_buffer,
            voxel_cross_points_buffer: voxel_cross_points_dynamic_buffer,
            mesh_vertex_map_buffer: mesh_vertex_map_dynamic_buffer,
            mesh_vertices_buffer: mesh_vertices_dynamic_buffer,
            mesh_indices_buffer: mesh_indices_dynamic_buffer,
            mesh_vertices_indices_count_buffer: mesh_vertices_indices_count_dynamic_buffer,
            terrain_chunk_buffer_bindings_map: HashMap::new(),
            should_create: false,
        }
    }

    /**
     * 设置各个动态 buffer 的 stride 大小 (单个实例的大小)
     */
    pub fn set_stride(&mut self, terrain_setting: &TerrainSetting) {
        // 沿着正方向拓宽一个voxel，用于填充缝隙。否则，chunk 与 chunk 会有一个缝隙。
        let voxel_num = terrain_setting.get_voxel_count_in_compute();
        let voxel_vertex_num = voxel_num + 1;

        let total_voxel_num = (voxel_num * voxel_num * voxel_num) as u64;
        // TODO: 对于重复创建的chunk，也许可以复用之前的顶点的数量，从而减少内存申请。
        let mesh_vertex_num = total_voxel_num;
        let total_voxel_vertex_num =
            (voxel_vertex_num * voxel_vertex_num * voxel_vertex_num) as u64;

        self.voxel_vertex_values_buffer.set_stride(
            BufferSize::new(f32::min_size().get() * total_voxel_vertex_num)
                .expect("Failed to set stride for voxel vertex values dynamic buffer"),
        );
        self.voxel_cross_points_buffer.set_stride(
            BufferSize::new(VoxelEdgeCrossPoint::min_size().get() * total_voxel_vertex_num * 3)
                .expect("Failed to set stride for voxel cross points dynamic buffer"),
        );
        self.mesh_vertex_map_buffer.set_stride(
            BufferSize::new(u32::min_size().get() * total_voxel_num)
                .expect("Failed to set stride for mesh vertex map dynamic buffer"),
        );
        self.mesh_vertices_buffer.set_stride(
            BufferSize::new(TerrainChunkVertexInfo::min_size().get() * mesh_vertex_num)
                .expect("Failed to set stride for mesh vertices dynamic buffer"),
        );
        self.mesh_indices_buffer.set_stride(
            BufferSize::new(u32::min_size().get() * mesh_vertex_num * 18)
                .expect("Failed to set stride for mesh indices dynamic buffer"),
        );
    }

    pub fn reserve_buffers(&mut self, context: &TerrainChunkMeshBufferReserveContext) {
        let _span = info_span!("terrain chunk main buffers reverse").entered();

        {
            let _span = info_span!("terrain chunk main info buffers reserve").entered();
            if self
                .terrain_chunk_info_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.should_create = true;
            }
        }

        {
            let _span =
                info_span!("terrain chunk main voxel vertex values buffers reserve").entered();
            if self
                .voxel_vertex_values_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.should_create = true;
            }
        }

        {
            let _span =
                info_span!("terrain chunk main voxel cross point buffers reserve").entered();
            if self
                .voxel_cross_points_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.should_create = true;
            }
        };

        {
            let _span = info_span!("terrain chunk main mesh vertex map buffers reserve").entered();
            if self
                .mesh_vertex_map_buffer
                .reserve_buffer(context.instance_num, context.render_device)
            {
                self.should_create = true;
            }
        };

        {
            let _span = info_span!("terrain chunk main mesh vertex stage buffers create").entered();
            if self
                .mesh_vertices_buffer
                .reserve_buffer(context.render_device, context.instance_num)
            {
                self.should_create = true;
            }
        };

        {
            let _span =
                info_span!("terrain chunk main mesh indices stage buffers create").entered();
            if self
                .mesh_indices_buffer
                .reserve_buffer(context.render_device, context.instance_num)
            {
                self.should_create = true;
            }
        };

        {
            let _span =
                info_span!("terrain chunk main mesh vertices indices count stage buffers reserve")
                    .entered();
            if self
                .mesh_vertices_indices_count_buffer
                .reserve_buffer(context.render_device, context.instance_num)
            {
                self.should_create = true;
            }
        };
    }

    pub fn set_buffers_data(&mut self, context: TerrainChunkMeshBufferCreateContext) {
        let _span = info_span!("terrain chunk main buffers write buffers data").entered();

        let voxel_size = context.terrain_setting.get_voxel_size();
        let chunk_size = context.terrain_setting.get_chunk_size();
        let voxel_num = context.terrain_setting.get_voxel_count_in_compute();

        let chunk_min = context.terrain_chunk_coord.to_world_pos(chunk_size);

        {
            let _span = info_span!("terrain chunk main chunk info buffers write buffer").entered();

            // 重置 chunk 信息
            self.terrain_chunk_info_buffer.push(&TerrainChunkInfo {
                chunk_min_location_size: Vec4::new(chunk_min.x, chunk_min.y, chunk_min.z, 0.0),
                voxel_size,
                voxel_num,
                qef_threshold: context.terrain_setting.qef_solver_threshold,
                qef_stddev: context.terrain_setting.qef_stddev,
            });
        }

        {
            let _span =
                info_span!("terrain chunk main vertices indices count buffers write buffer")
                    .entered();
            // 重置顶点和索引数量为0
            self.mesh_vertices_indices_count_buffer
                .push_value(TerrainChunkVerticesIndicesCount {
                    vertices_count: 0,
                    indices_count: 0,
                });
        }
    }

    pub fn write_dynamic_buffers(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
    ) {
        if self
            .terrain_chunk_info_buffer
            .write_buffer(render_device, render_queue)
        {
            self.should_create = true;
        }

        self.mesh_vertices_indices_count_buffer
            .write_buffer(render_device, render_queue);
    }

    pub fn stage_buffers(&self, command_encoder: &mut CommandEncoder) {
        self.mesh_vertices_buffer.stage_buffer(
            command_encoder,
            self.mesh_vertices_buffer
                .cpu_buffer
                .as_ref()
                .expect("Mesh vertices CPU buffer should exist")
                .size(),
        );
        self.mesh_indices_buffer.stage_buffer(
            command_encoder,
            self.mesh_indices_buffer
                .cpu_buffer
                .as_ref()
                .expect("Mesh indices CPU buffer should exist")
                .size(),
        );
        self.mesh_vertices_indices_count_buffer.stage_buffer(
            command_encoder,
            self.mesh_vertices_indices_count_buffer
                .cpu_buffer
                .as_ref()
                .expect("Mesh vertices indices count CPU buffer should exist")
                .size(),
        );
    }

    pub fn unmap(&self) {
        self.mesh_vertices_buffer.unmap();
        self.mesh_indices_buffer.unmap();
        self.mesh_vertices_indices_count_buffer.unmap();
    }

    pub fn map_async(&self) {
        self.mesh_vertices_buffer.map_async(..);
        self.mesh_indices_buffer.map_async(..);
        self.mesh_vertices_indices_count_buffer.map_async(..);
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
            size: Some(
                BufferSize::new(type_size * current_num as u64)
                    .expect("Buffer size should be valid"),
            ),
        }
    }

    pub fn get_right_offset(&self) -> BufferAddress {
        self.offset + self.size.expect("Buffer size should exist").get()
    }
}

#[derive(Default, Debug)]
pub struct TerrainChunkMeshBufferBindings {
    pub terrain_chunk_info_buffer_binding: DynamicBufferBindingInfo,
    pub voxel_vertex_values_buffer_binding: DynamicBufferBindingInfo,
    pub voxel_cross_points_buffer_binding: DynamicBufferBindingInfo,

    pub mesh_vertex_map_buffer_binding: DynamicBufferBindingInfo,

    pub mesh_vertices_buffer_binding: DynamicBufferBindingInfo,
    pub mesh_indices_buffer_binding: DynamicBufferBindingInfo,

    pub mesh_vertices_indices_count_buffer_binding: DynamicBufferBindingInfo,
}

pub struct TerrainChunkMeshBufferCreateContext<'a> {
    pub terrain_chunk_coord: TerrainChunkCoord,
    pub terrain_setting: &'a TerrainSetting,
    pub render_entity: Entity,
}

pub struct TerrainChunkMeshBufferBindingsBuilder<'a> {
    pub current_index: usize,

    pub terrain_setting: &'a TerrainSetting,
    pub mesh_buffers: &'a TerrainChunkMeshBuffers,
}

impl TerrainChunkMeshBufferBindings {
    pub fn get_dynamic_offset(&self) -> Vec<DynamicOffset> {
        vec![
            self.terrain_chunk_info_buffer_binding.offset as DynamicOffset,
            self.voxel_vertex_values_buffer_binding.offset as DynamicOffset,
            self.voxel_cross_points_buffer_binding.offset as DynamicOffset,
            self.mesh_vertices_buffer_binding.offset as DynamicOffset,
            self.mesh_indices_buffer_binding.offset as DynamicOffset,
            self.mesh_vertex_map_buffer_binding.offset as DynamicOffset,
            self.mesh_vertices_indices_count_buffer_binding.offset as DynamicOffset,
        ]
    }

    pub fn rebuild_binding_size(&mut self, builder: TerrainChunkMeshBufferBindingsBuilder) {
        let last_index = builder.current_index - 1;

        self.terrain_chunk_info_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .mesh_buffers
                .terrain_chunk_info_buffer
                .get_alignment(),
        );

        self.voxel_vertex_values_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .mesh_buffers
                .voxel_vertex_values_buffer
                .get_stride_alignment(),
        );

        self.voxel_cross_points_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .mesh_buffers
                .voxel_cross_points_buffer
                .get_stride_alignment(),
        );

        self.mesh_vertex_map_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .mesh_buffers
                .mesh_vertex_map_buffer
                .get_stride_alignment(),
        );

        self.mesh_vertices_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder.mesh_buffers.mesh_vertices_buffer.get_alignment(),
        );

        self.mesh_indices_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder.mesh_buffers.mesh_indices_buffer.get_alignment(),
        );

        self.mesh_vertices_indices_count_buffer_binding = DynamicBufferBindingInfo::from_num(
            last_index,
            1,
            builder
                .mesh_buffers
                .mesh_vertices_indices_count_buffer
                .get_alignment(),
        );
    }
}

pub(crate) fn prepare_mesh_buffers(
    query: Query<(Entity, &TerrainChunkCoord, &TerrainChunkMeshingState)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    terrain_setting: Res<TerrainSetting>,
    mut mesh_buffers: ResMut<TerrainChunkMeshBuffers>,
) {
    mesh_buffers.clear();

    mesh_buffers.set_stride(&terrain_setting);

    let mut meshing_chunk_count = 0;
    for (_entity, _coord, state) in query.iter() {
        if *state != TerrainChunkMeshingState::Meshing {
            continue;
        }

        meshing_chunk_count += 1;
    }

    if meshing_chunk_count == 0 {
        return;
    }

    let context = TerrainChunkMeshBufferReserveContext {
        render_device: &render_device,
        render_queue: &render_queue,
        terrain_setting: &terrain_setting,
        instance_num: meshing_chunk_count,
    };
    mesh_buffers.reserve_buffers(&context);

    for (entity, coord, state) in query.iter() {
        if state != &TerrainChunkMeshingState::Meshing {
            continue;
        }

        // 为每个正在网格化的 Chunk 准备缓冲区绑定信息
        let mut buffer_bindings = TerrainChunkMeshBufferBindings::default();
        let builder = TerrainChunkMeshBufferBindingsBuilder {
            current_index: meshing_chunk_count,
            terrain_setting: &terrain_setting,
            mesh_buffers: &mesh_buffers,
        };
        buffer_bindings.rebuild_binding_size(builder);
        mesh_buffers.insert_terrain_chunk_buffer_bindings(entity, buffer_bindings);

        let context = TerrainChunkMeshBufferCreateContext {
            terrain_chunk_coord: *coord,
            terrain_setting: &terrain_setting,
            render_entity: entity,
        };
        mesh_buffers.set_buffers_data(context);
    }

    mesh_buffers.write_dynamic_buffers(&render_device, &render_queue);
}
