use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume},
        Vec3A,
    },
    prelude::*,
    render::extract_resource::ExtractResource,
    utils::HashMap,
};

use crate::{
    chunk_mgr::chunk_loader::{LeafNodeKey, TerrainChunkLoader},
    isosurface::dc::gpu_dc::buffer_cache::TerrainChunkCSGOperation,
    lod::{lod_octree::TerrainLodOctree, morton_code::MortonCode},
    setting::TerrainSetting,
};

#[derive(Event, Debug, Clone, Copy)]
pub struct CSGOperateApplyEvent {
    pub transform: Transform,
    pub primitive: CSGPrimitive,
    pub operate_type: CSGOperateType,
}

impl CSGOperateApplyEvent {
    pub fn to_gpu_type(&self) -> TerrainChunkCSGOperation {
        TerrainChunkCSGOperation {
            location: self.transform.translation,
            primitive_type: self.primitive.to_index(),
            shape: self.primitive.to_shape(),
            operation_type: self.operate_type.to_index(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSGOperateType {
    Round,
    Union,
    Difference,
    Intersection,
    SmoothUnion,
    SmoothDifference,
    SmoothIntersection,
}

impl CSGOperateType {
    pub fn to_index(&self) -> u32 {
        match self {
            CSGOperateType::Round => 0,
            CSGOperateType::Union => 1,
            CSGOperateType::Difference => 2,
            CSGOperateType::Intersection => 3,
            CSGOperateType::SmoothUnion => 4,
            CSGOperateType::SmoothDifference => 5,
            CSGOperateType::SmoothIntersection => 6,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CSGPrimitive {
    Sphere { radius: f32 },
    Box { size: Vec3 },
}

impl CSGPrimitive {
    pub fn to_index(&self) -> u32 {
        match self {
            CSGPrimitive::Sphere { radius: _ } => 0,
            CSGPrimitive::Box { size: _ } => 1,
        }
    }

    pub fn to_shape(&self) -> Vec3 {
        match self {
            CSGPrimitive::Sphere { radius } => Vec3::splat(*radius),
            CSGPrimitive::Box { size } => *size,
        }
    }
}

impl CSGPrimitive {
    pub fn aabb(&self, transform: &Transform) -> Aabb3d {
        match self {
            CSGPrimitive::Sphere { radius } => {
                let radius = *radius;
                let center = transform.translation;
                Aabb3d::new(center - Vec3::splat(radius), center + Vec3::splat(radius))
            }
            CSGPrimitive::Box { size } => {
                let size = *size;
                let center = transform.translation;
                Aabb3d::new(center - size / 2.0, center + size / 2.0)
            }
        }
    }
}

// 支持队列，用于撤销。
// 保持执行顺序，
// 切分chunk，对每个chunk进行上传数据。

#[derive(Resource, Debug, Default, ExtractResource, Clone)]
pub struct CSGOperationRecords {
    pub operation: Vec<CSGOperateApplyEvent>,
    /// key is lod octree node code, value(usize) is operation index
    pub chunk_map: HashMap<MortonCode, Vec<usize>>,
}

impl CSGOperationRecords {
    pub fn get_chunk_gpu_data(&self, code: MortonCode) -> Option<Vec<TerrainChunkCSGOperation>> {
        match self.chunk_map.get(&code) {
            Some(operation_index) => {
                let mut result = Vec::with_capacity(operation_index.len());
                for index in operation_index {
                    let event = self.operation[*index];
                    result.push(event.to_gpu_type());
                }
                Some(result)
            }
            None => None,
        }
    }
}

pub fn read_csg_operation_apply_event(
    mut event_reader: EventReader<CSGOperateApplyEvent>,
    mut csg_operation_records: ResMut<CSGOperationRecords>,
    mut loader: ResMut<TerrainChunkLoader>,
    lod_octree: Res<TerrainLodOctree>,
    terrain_setting: Res<TerrainSetting>,
) {
    for event in event_reader.read() {
        let index = csg_operation_records.operation.len();
        let mut aabb = event.primitive.aabb(&event.transform);

        // 用于判断是否缝隙是否需要remesh。
        // 如果在地图外，则不考虑，应该不会有这种情况。
        if let Some(located_node) =
            lod_octree.get_node_by_location(event.transform.translation.into(), &terrain_setting)
        {
            let voxel_size = terrain_setting.get_voxel_size(located_node.code.depth);
            aabb = aabb.grow(Vec3A::splat(voxel_size));
        }

        info!("apply_csg_operation: {:?}", event);

        let intersect_nodes = lod_octree.get_intersect_nodes(aabb, &terrain_setting);
        for node in intersect_nodes {
            let leaf_node_key = LeafNodeKey::from_lod_leaf_node(&node);
            loader.insert_pending_reload_leaf_node_map(node.code, leaf_node_key);

            let entry = csg_operation_records
                .chunk_map
                .entry(node.code)
                .or_insert(vec![]);
            entry.push(index);
        }

        csg_operation_records.operation.push(*event);

        info!("apply_csg_operation: {:?}", csg_operation_records);
    }
}
