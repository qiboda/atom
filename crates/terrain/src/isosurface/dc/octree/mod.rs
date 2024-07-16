pub mod address;
pub mod cell;
pub mod tables;
pub mod vertex;

use core::panic;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use address::DepthCoordMap;
use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    prelude::*,
    utils::HashMap,
};

use cell::CellType;
use ndshape::Shape;
use pqef::Quadric;
use strum::{EnumCount, IntoEnumIterator};
use tables::VertexIndex;

use {address::CellAddress, cell::Cell};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum OctreeSubdivisionResult {
    Subdivided,
    MaxDepth,
    NotSubdivided,
}

pub trait OctreeBranchPolicy {
    fn check_to_subdivision(&self, aabb: Aabb3d) -> OctreeSubdivisionResult;
}

pub trait OctreeSampler {
    fn sampler(&self, loc: Vec3) -> f32;
    fn sampler_split(&self, x: f32, y: f32, z: f32) -> f32;
}

pub trait OctreeContext: OctreeBranchPolicy + OctreeSampler {}

/// octree
/// 1. 细分octree，是否可以细分
/// 2. 计算叶子的qef。
/// 3. 反向进行收缩，节省内容空间。
///
/// 自顶向下，还是自底向上？
///
/// 也许可以通过不断插入顶点，来决定是否应该细分。()
/// 性能比每次细分来检测所有的顶点数据要好（层级细分是层级数*(每层涵盖的顶点数 = 所有顶点数）)，
#[derive(Component)]
pub struct Octree {
    pub address_cell_map: Arc<RwLock<HashMap<CellAddress, Cell>>>,
    pub cell_shape: ndshape::RuntimeShape<u32, 3>,
}

#[derive(Debug)]
pub struct OctreeProxy<'a> {
    pub cell_addresses: RwLockReadGuard<'a, HashMap<CellAddress, Cell>>,
}

impl Octree {
    pub fn new(cell_shape: ndshape::RuntimeShape<u32, 3>) -> Self {
        Octree {
            address_cell_map: Arc::new(RwLock::new(HashMap::default())),
            cell_shape,
        }
    }

    pub fn get_octree_depth(cell_shape: &ndshape::RuntimeShape<u32, 3>) -> u16 {
        let cell_shape_size = cell_shape.as_array();
        (cell_shape_size[0] as f32).log2().ceil() as u16 + 1
    }

    /// size: is the size of the sampler_data
    #[allow(clippy::too_many_arguments)]
    pub fn build_bottom_up<S>(
        octree: &mut Octree,
        sampler_data: &[f32],
        shape: &S,
        voxel_size: f32,
        std_dev_pos: f32,
        std_dev_normal: f32,
        octree_offset: Vec3,
        sampler_source: &impl OctreeSampler,
        cell_address: Arc<RwLock<HashMap<u16, Vec<CellAddress>>>>,
    ) where
        S: ndshape::Shape<3, Coord = u32>,
    {
        let size = shape.as_array();
        assert_eq!(size[0], size[1]);
        assert_eq!(size[0], size[2]);
        assert_eq!(
            ((size[0]) as f32).log2().ceil(),
            ((size[0] - 1) as f32).log2().ceil() + 1.0
        );

        let cell_address = cell_address.read().unwrap();
        let cell_shape_size = octree.cell_shape.as_array();

        let depth = Octree::get_octree_depth(&octree.cell_shape);
        let Some(leaf_address_mapper) = cell_address.get(&depth) else {
            panic!(
                "depth {} is invalid! and cell_mapper size: {}, and keys:{:?}",
                depth,
                cell_address.len(),
                cell_address.keys()
            );
        };

        let mut address_cell_map = octree.address_cell_map.write().unwrap();

        // build children leaf cell
        for x in 0..cell_shape_size[0] {
            for y in 0..cell_shape_size[1] {
                for z in 0..cell_shape_size[2] {
                    let mut conner_sampler_data = [0.0; 8];

                    for i in VertexIndex::iter() {
                        let offset = i.to_array();
                        let index = shape.linearize([x + offset[0], y + offset[1], z + offset[2]]);
                        conner_sampler_data[i as usize] = sampler_data[index as usize];
                    }

                    if conner_sampler_data.iter().all(|v| *v < 0.0) {
                        continue;
                    }

                    if conner_sampler_data.iter().all(|v| *v >= 0.0) {
                        continue;
                    }

                    let cell_index = octree.cell_shape.linearize([x, y, z]);
                    let cell_address = leaf_address_mapper[cell_index as usize];

                    debug!(
                    "leaf coord:{}, {}, {}, conner_sampler_data: {:?}, address: {:?}, cell_index:{}",
                    x, y, z, conner_sampler_data, cell_address, cell_index);

                    let mut cell = Cell::new(CellType::Leaf, cell_address);

                    let cell_size = voxel_size;
                    let cell_half_size = voxel_size * 0.5;
                    cell.coord = Vec3A::new(x as f32, y as f32, z as f32);
                    cell.aabb = Aabb3d::new(
                        Vec3::new(
                            x as f32 * cell_size + cell_half_size,
                            y as f32 * cell_size + cell_half_size,
                            z as f32 * cell_size + cell_half_size,
                        ) + octree_offset,
                        Vec3::splat(cell_half_size),
                    );
                    cell.estimate_vertex(
                        sampler_source,
                        conner_sampler_data,
                        std_dev_pos,
                        std_dev_normal,
                    );

                    address_cell_map.insert(cell_address, cell);
                }
            }
        }

        debug!("leaf octree.cell_addresses len: {}", address_cell_map.len());

        let mut cell_shape_size = octree.cell_shape.as_array();
        cell_shape_size = [
            cell_shape_size[0] / 2,
            cell_shape_size[1] / 2,
            cell_shape_size[2] / 2,
        ];
        let mut cell_shape = ndshape::RuntimeShape::<u32, 3>::new([
            cell_shape_size[0],
            cell_shape_size[1],
            cell_shape_size[2],
        ]);

        for i in (1..depth).rev() {
            trace!("depth: {}", i);
            let cell_address_mapper = cell_address.get(&i).unwrap();
            for x in 0..cell_shape_size[0] {
                for y in 0..cell_shape_size[1] {
                    for z in 0..cell_shape_size[2] {
                        let cell_index = cell_shape.linearize([x, y, z]);
                        let cell_address = cell_address_mapper[cell_index as usize];

                        let mut exist_child = false;
                        let mut all_children_leaf = true;
                        let mut mid_mat = None;
                        let mut cell_mats = [None; VertexIndex::COUNT];
                        for (children_index, child_address) in
                            cell_address.get_children_addresses().iter().enumerate()
                        {
                            let child_cell = address_cell_map.get(child_address);
                            if let Some(child_cell) = child_cell {
                                exist_child = true;
                                match child_cell.cell_type {
                                    CellType::Branch => {
                                        all_children_leaf = false;
                                    }
                                    CellType::Leaf => {
                                        // child cell的children_index对角的点的材质
                                        mid_mat =
                                            Some(child_cell.vertices_mat_types[7 - children_index]);
                                        cell_mats[children_index] =
                                            Some(child_cell.vertices_mat_types[children_index]);
                                    }
                                }
                            }
                        }

                        let cell_size = voxel_size * 2.0f32.powi((depth - i) as i32);
                        let cell_half_size = cell_size * 0.5;
                        if exist_child {
                            let mut cell = Cell::new(CellType::Branch, cell_address);
                            cell.coord = Vec3A::new(x as f32, y as f32, z as f32);
                            cell.aabb = Aabb3d::new(
                                Vec3::new(
                                    x as f32 * cell_size + cell_half_size,
                                    y as f32 * cell_size + cell_half_size,
                                    z as f32 * cell_size + cell_half_size,
                                ) + octree_offset,
                                Vec3::splat(cell_half_size),
                            );
                            trace!("depth: {}, aabb half size: {}", i, cell_half_size);
                            if all_children_leaf {
                                for (vertex_index, _) in VertexIndex::iter().enumerate() {
                                    match cell_mats[vertex_index] {
                                        Some(mat) => cell.vertices_mat_types[vertex_index] = mat,
                                        None => {
                                            cell.vertices_mat_types[vertex_index] = mid_mat.unwrap()
                                        }
                                    }
                                }
                            }
                            address_cell_map.insert(cell_address, cell);
                        }
                    }
                }
            }

            cell_shape_size = cell_shape.as_array();
            cell_shape_size = [
                cell_shape_size[0] / 2,
                cell_shape_size[1] / 2,
                cell_shape_size[2] / 2,
            ];
            cell_shape = ndshape::RuntimeShape::<u32, 3>::new([
                cell_shape_size[0],
                cell_shape_size[1],
                cell_shape_size[2],
            ]);

            debug!(
                "depth {}, cell_addresses len: {}",
                i,
                address_cell_map.len()
            );
        }
    }

    pub fn simplify_octree(
        address_cell_map: Arc<RwLock<HashMap<CellAddress, Cell>>>,
        cell_shape: ndshape::RuntimeShape<u32, 3>,
        deep_coord_mapper: Arc<RwLock<DepthCoordMap>>,
        qef_threshold_map: HashMap<u16, f32>,
    ) {
        let mut address_cell_map = address_cell_map.write().unwrap();
        let deep_coord_mapper = deep_coord_mapper.read().unwrap();
        debug!("leaf octree.cell_addresses len: {}", address_cell_map.len());

        let depth = Octree::get_octree_depth(&cell_shape);

        let mut cell_shape_size = cell_shape.as_array();
        cell_shape_size = [
            cell_shape_size[0] / 2,
            cell_shape_size[1] / 2,
            cell_shape_size[2] / 2,
        ];
        let mut cell_shape = ndshape::RuntimeShape::<u32, 3>::new([
            cell_shape_size[0],
            cell_shape_size[1],
            cell_shape_size[2],
        ]);

        for i in (1..depth).rev() {
            trace!("depth: {}", i);
            let coord_address_vec = deep_coord_mapper.get(&i).unwrap();
            let qef_threshold = qef_threshold_map.get(&i).unwrap_or(&0.1);
            for x in 0..cell_shape_size[0] {
                for y in 0..cell_shape_size[1] {
                    for z in 0..cell_shape_size[2] {
                        let cell_index = cell_shape.linearize([x, y, z]);
                        let cell_address = coord_address_vec[cell_index as usize];

                        if address_cell_map.get_mut(&cell_address).is_none() {
                            continue;
                        }

                        let mut all_children_leaf = true;
                        let mut leaf_children_count = 0;
                        let mut qef = Quadric::default();
                        let mut avg_normal = Vec3A::ZERO;
                        let mut avg_position = Vec3::ZERO;

                        for child_address in cell_address.get_children_addresses().iter() {
                            let child_cell = address_cell_map.get(child_address);
                            if let Some(child_cell) = child_cell {
                                match child_cell.cell_type {
                                    CellType::Branch => {
                                        all_children_leaf = false;
                                    }
                                    CellType::Leaf => {
                                        leaf_children_count += 1;
                                        if let Some(child_qef) = child_cell.qef {
                                            qef += child_qef;
                                            avg_normal += child_cell.normal_estimate;
                                            // 应该没用，因为有误差限制，不应该允许超出Cell的误差限制。
                                            avg_position += child_cell.vertex_estimate;
                                        }
                                    }
                                }
                            }
                        }

                        let Some(cell) = address_cell_map.get_mut(&cell_address) else {
                            continue;
                        };

                        if all_children_leaf {
                            avg_normal /= leaf_children_count as f32;
                            avg_position /= leaf_children_count as f32;
                            cell.estimate_vertex_with_qef(
                                qef,
                                avg_position.into(),
                                avg_normal.normalize(),
                            );
                            debug!( "cell, vertex: {}, qef error {}, {}, coord:{:?}, leaf_children_count: {}", cell.vertex_estimate, cell.qef_error, cell.address, cell.coord, leaf_children_count);
                            if cell.qef_error < *qef_threshold {
                                cell.cell_type = CellType::Leaf;

                                for child_address in cell_address.get_children_addresses() {
                                    address_cell_map.remove(&child_address);
                                }
                            }
                        }
                    }
                }
            }

            cell_shape_size = cell_shape.as_array();
            cell_shape_size = [
                cell_shape_size[0] / 2,
                cell_shape_size[1] / 2,
                cell_shape_size[2] / 2,
            ];
            cell_shape = ndshape::RuntimeShape::<u32, 3>::new([
                cell_shape_size[0],
                cell_shape_size[1],
                cell_shape_size[2],
            ]);

            debug!(
                "depth {}, cell_addresses len: {}",
                i,
                address_cell_map.len()
            );
        }
    }
}

impl Octree {
    pub fn check_children_relation(cell_addresses: Arc<RwLock<HashMap<CellAddress, Cell>>>) {
        let cell_addresses = cell_addresses.read().unwrap();
        cell_addresses
            .iter()
            .for_each(|(address, cell)| match cell.cell_type {
                CellType::Branch => {
                    assert_eq!(cell.address, *address);
                    let mut exist_child = false;
                    for child_address in cell.address.get_children_addresses() {
                        if cell_addresses.get(&child_address).is_some() {
                            exist_child = true;
                            break;
                        }
                    }
                    assert!(exist_child);
                }
                CellType::Leaf => {
                    assert_eq!(cell.address, *address);
                    for child_address in cell.address.get_children_addresses() {
                        assert!(cell_addresses.get(&child_address).is_none());
                    }
                }
            });
    }
}
