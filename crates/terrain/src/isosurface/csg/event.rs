use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume, IntersectsVolume},
        Vec3A,
    },
    prelude::*,
    render::extract_resource::ExtractResource,
    utils::HashMap,
};

use crate::{
    chunk_mgr::chunk_loader::{LeafNodeKey, TerrainChunkLoader},
    isosurface::dc::gpu_dc::buffer_type::TerrainChunkCSGOperation,
    lod::{
        lod_octree::{TerrainLodOctree, TerrainLodOctreeNode},
        morton_code::MortonCode,
    },
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
        let shape = self.primitive.to_shape();
        TerrainChunkCSGOperation {
            location: Vec3::new(
                self.transform.translation.x,
                self.transform.translation.y,
                self.transform.translation.z,
            ),
            primitive_type: self.primitive.to_index(),
            shape: Vec3::new(shape.x, shape.y, shape.z),
            operation_type: self.operate_type.to_index(),
        }
        // TerrainChunkCSGOperation {
        //     location: self.transform.translation,
        //     primitive_type: self.primitive.to_index(),
        //     shape: self.primitive.to_shape(),
        //     operation_type: self.operate_type.to_index(),
        // }
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
                Aabb3d::new(center, Vec3::splat(radius))
            }
            CSGPrimitive::Box { size } => {
                let size = *size;
                let center = transform.translation;
                Aabb3d::new(center, size / 2.0)
            }
        }
    }
}

// 支持队列，用于撤销。
// 保持执行顺序，
// 切分chunk，对每个chunk进行上传数据。

#[derive(Resource, Debug, Default, ExtractResource, Clone)]
pub struct CSGOperationRecords {
    pub operations: Vec<CSGOperateApplyEvent>,
    /// key is lod octree node code, value(usize) is operation index
    pub chunk_map: HashMap<MortonCode, Vec<usize>>,
}

impl CSGOperationRecords {
    pub fn get_chunk_gpu_data(&self, code: MortonCode) -> Option<Vec<TerrainChunkCSGOperation>> {
        match self.chunk_map.get(&code) {
            Some(operation_index) => {
                let mut result = Vec::with_capacity(operation_index.len());
                for index in operation_index {
                    let event = self.operations[*index];
                    result.push(event.to_gpu_type());
                }
                Some(result)
            }
            None => None,
        }
    }

    pub fn insert_node(&mut self, node: &TerrainLodOctreeNode, index: usize) {
        let entry = self.chunk_map.entry(node.code).or_default();
        entry.push(index);
    }
}

pub fn update_csg_operations_records(
    mut csg_operation_records: ResMut<CSGOperationRecords>,
    lod_octree: Res<TerrainLodOctree>,
    terrain_setting: Res<TerrainSetting>,
) {
    let csg_operations = std::mem::take(&mut csg_operation_records.operations);

    for level in lod_octree.octree_levels.iter() {
        for node in level.get_added_nodes() {
            for (index, operation) in csg_operations.iter().enumerate() {
                let mut aabb = operation.primitive.aabb(&operation.transform);

                // 用于判断是否缝隙是否需要remesh。
                // 如果在地图外，则不考虑，应该不会有这种情况。
                if let Some(located_node) = lod_octree
                    .get_node_by_location(operation.transform.translation.into(), &terrain_setting)
                {
                    let voxel_size = terrain_setting.get_voxel_size(located_node.code.depth);
                    // 可能aabb的边界刚好在最外层voxel的最里面，因此乘以2.0
                    aabb = aabb.grow(Vec3A::splat(voxel_size * 2.0));
                } else {
                    warn!("csg location can not get lod octree node: {:?}", operation);
                }

                if node.aabb.intersects(&aabb) {
                    // 不需要插入，因为改变lod node的部分会进行创建。
                    // let leaf_node_key = LeafNodeKey::from_lod_leaf_node(&node);
                    // loader.insert_pending_reload_leaf_node_map(node.code, leaf_node_key);

                    csg_operation_records.insert_node(node, index);
                }
            }

            for node in level.get_removed_nodes() {
                csg_operation_records.chunk_map.remove(&node.code);
            }
        }
    }

    csg_operation_records.operations = csg_operations;
}

pub fn read_csg_operation_apply_event(
    mut event_reader: EventReader<CSGOperateApplyEvent>,
    mut csg_operation_records: ResMut<CSGOperationRecords>,
    mut loader: ResMut<TerrainChunkLoader>,
    lod_octree: Res<TerrainLodOctree>,
    terrain_setting: Res<TerrainSetting>,
) {
    for event in event_reader.read() {
        let index = csg_operation_records.operations.len();
        let mut aabb = event.primitive.aabb(&event.transform);

        // 用于判断是否缝隙是否需要remesh。
        // 如果在地图外，则不考虑，应该不会有这种情况。
        if let Some(located_node) =
            lod_octree.get_node_by_location(event.transform.translation.into(), &terrain_setting)
        {
            let voxel_size = terrain_setting.get_voxel_size(located_node.code.depth);
            debug!("csg aabb grow voxel size: {}", voxel_size);
            // aabb扩展范围收到csg操作的范围的影响，因此先乘以2.0看看效果。
            aabb = aabb.grow(Vec3A::splat(voxel_size * 2.0));
        } else {
            warn!("csg location can not get lod octree node: {:?}", event);
        }

        let intersect_nodes = lod_octree.get_intersect_nodes(aabb, &terrain_setting);
        if intersect_nodes.is_empty() {
            warn!("csg operation {:?} no intersect node", event);
            continue;
        }

        debug!("intersect_nodes: {:?} aabb: {:?}", intersect_nodes, aabb);

        for node in intersect_nodes {
            let leaf_node_key = LeafNodeKey::from_lod_leaf_node(&node);
            loader.insert_pending_reload_leaf_node_map(node.code, leaf_node_key);

            csg_operation_records.insert_node(&node, index);
        }

        csg_operation_records.operations.push(*event);
    }
}
