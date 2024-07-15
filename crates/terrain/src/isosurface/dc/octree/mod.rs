pub mod address;
pub mod cell;
pub mod tables;
pub mod vertex;

use core::panic;
use std::sync::{Arc, RwLock, RwLockReadGuard};

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
#[derive(Debug, Component, Default)]
pub struct Octree {
    pub cell_addresses: Arc<RwLock<HashMap<CellAddress, Cell>>>,
    pub lod: u8,

    pub leaf_cells: Vec<CellAddress>,
}

#[derive(Debug)]
pub struct OctreeProxy<'a> {
    pub cell_addresses: RwLockReadGuard<'a, HashMap<CellAddress, Cell>>,
}

/// size: is the size of the sampler_data
pub fn build_bottom_up<S>(
    octree: &mut Octree,
    sampler_data: &[f32],
    shape: &S,
    voxel_size: f32,
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

    let cell_shape = ndshape::RuntimeShape::<u32, 3>::new([size[0] - 1, size[1] - 1, size[2] - 1]);

    let cell_address = cell_address.read().unwrap();
    let cell_shape_size = cell_shape.as_array();

    let depth = ((size[0] - 1) as f32).log2().ceil() as u16 + 1;
    let Some(leaf_address_mapper) = cell_address.get(&depth) else {
        panic!(
            "depth {} is invalid! and cell_mapper size: {}, and keys:{:?}",
            depth,
            cell_address.len(),
            cell_address.keys()
        );
    };

    let mut cell_addresses = octree.cell_addresses.write().unwrap();

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

                let cell_index = cell_shape.linearize([x, y, z]);
                let cell_address = leaf_address_mapper[cell_index as usize];

                debug!(
                    "leaf coord:{}, {}, {}, conner_sampler_data: {:?}, address: {:?}, cell_index:{}",
                    x, y, z, conner_sampler_data, cell_address, cell_index
                );
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
                cell.vertices_samples = conner_sampler_data;
                cell.estimate_vertex(sampler_source, 1.0);

                cell_addresses.insert(cell_address, cell);
            }
        }
    }

    info!("leaf octree.cell_addresses len: {}", cell_addresses.len());

    // TODO: 依次构建parent cell。
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
        let cell_address_mapper = cell_address.get(&i).unwrap();
        for x in 0..cell_shape_size[0] {
            for y in 0..cell_shape_size[1] {
                for z in 0..cell_shape_size[2] {
                    let cell_index = cell_shape.linearize([x, y, z]);
                    let cell_address = cell_address_mapper[cell_index as usize];

                    let mut exist_child = false;
                    let mut all_children_leaf = true;
                    let mut leaf_children_count = 0;
                    let mut qef = Quadric::default();
                    let mut avg_normal = Vec3A::ZERO;
                    let mut avg_position = Vec3::ZERO;
                    let mut mid_mat = None;
                    let mut cell_mats = [None; VertexIndex::COUNT];
                    for (children_index, child_address) in
                        cell_address.get_children_addresses().iter().enumerate()
                    {
                        let child_cell = cell_addresses.get(child_address);
                        if let Some(child_cell) = child_cell {
                            exist_child = true;
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

                                        // child cell的children_index对角的点的材质
                                        mid_mat =
                                            Some(child_cell.vertices_mat_types[7 - children_index]);
                                        cell_mats[children_index] =
                                            Some(child_cell.vertices_mat_types[children_index]);
                                    }
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

                            avg_normal /= leaf_children_count as f32;
                            avg_position /= leaf_children_count as f32;
                            cell.estimate_vertex_with_qef(
                                qef,
                                avg_position.into(),
                                avg_normal.normalize(),
                            );
                            debug!(
                                "cell, vertex: {}, qef error {}, {}, coord:{:?}, leaf_children_count: {}",
                                cell.vertex_estimate, cell.qef_error, cell.address, cell.coord, leaf_children_count
                            );
                            if cell.qef_error < 0.01 {
                                cell.cell_type = CellType::Leaf;

                                for child_address in cell_address.get_children_addresses() {
                                    cell_addresses.remove(&child_address);
                                }
                            }
                        }
                        cell_addresses.insert(cell_address, cell);
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

        info!("depth {}, cell_addresses len: {}", i, cell_addresses.len());
    }

    // if cell_addresses.len() == 1 {
    //     cell_addresses.clear();
    // }
}
