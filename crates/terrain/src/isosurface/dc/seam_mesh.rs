use std::sync::{Arc, RwLock};

use bevy::{prelude::*, utils::hashbrown::HashMap};
use ndshape::{AbstractShape, RuntimeShape};

use crate::{
    chunk_mgr::{
        chunk::{
            bundle::TerrainChunk,
            chunk_lod::{LodType, OctreeDepthType, TerrainChunkAabb, TerrainChunkLod},
            state::TerrainChunkAddress,
        },
        chunk_loader::TerrainChunkLoader,
        chunk_mapper::TerrainChunkMapper,
    },
    isosurface::{
        comp::{SeamMeshState, TerrainChunkMainGenerator, TerrainChunkSeamGenerator},
        dc::octree::{
            check_octree_nodes_relation,
            tables::{EdgeIndex, FaceIndex, SubNodeIndex, VertexIndex},
        },
        mesh::mesh_info::MeshInfo,
        surface::shape_surface::{IsosurfaceContext, ShapeSurface},
    },
    lod::{
        lod_octree::{LodOctreeMap, LodOctreeNode},
        neighbor_query::{
            get_edge_neighbor_lod_octree_nodes, get_face_neighbor_lod_octree_nodes,
            get_vertex_neighbor_lod_octree_nodes,
        },
    },
    setting::TerrainSetting,
};

use super::{
    dual_contouring::{self, DefaultDualContouringVisiter},
    octree::{address::NodeAddress, node::Node, Octree, OctreeProxy},
    OctreeDepthCoordMapper,
};
use bevy_async_task::AsyncTaskPool;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
async fn construct_octree_task(
    entity: Entity,
    octrees: SeamNeighborOctrees,
    node_address_mapper: Arc<RwLock<HashMap<OctreeDepthType, Vec<NodeAddress>>>>,
    min_lod: LodType,
    min_voxel_size: f32,
    chunk_address: TerrainChunkAddress,
    chunk_aabb: TerrainChunkAabb,
) -> (Entity, Octree) {
    let _span = info_span!("seam mesh construct octree task", ?chunk_address, min_lod).entered();

    let new_chunk_size = (chunk_aabb.max.x - chunk_aabb.min.x) * 2.0;
    let new_chunk_voxel_num = (new_chunk_size / min_voxel_size) as u32;
    let new_chunk_depth = new_chunk_voxel_num.ilog2() as OctreeDepthType;
    let new_chunk_offset = chunk_aabb.min;

    let shape = RuntimeShape::<u32, 3>::new([
        new_chunk_voxel_num,
        new_chunk_voxel_num,
        new_chunk_voxel_num,
    ]);
    info!(
        "lod_voxel size: {}, chunk size: {}, voxel num: {}, chunk_depth: {}, chunk_offset: {}",
        min_voxel_size, new_chunk_size, new_chunk_voxel_num, new_chunk_depth, new_chunk_offset
    );

    let mut seam_leaf_nodes: HashMap<NodeAddress, Node> = HashMap::default();
    let mut get_seam_leaf_nodes =
        |subnode_index, octree: Arc<RwLock<HashMap<NodeAddress, Node>>>| {
            let node_map = octree.read().unwrap();
            if node_map.len() < 1 {
                return;
            }

            let leaf_nodes =
                Octree::get_all_seam_leaf_nodes(&node_map, chunk_aabb.0, subnode_index);
            trace!(
                "get leaf nodes: {} at {:?}",
                leaf_nodes.len(),
                subnode_index
            );

            let node_address_mapper = node_address_mapper.read().unwrap();
            for node in leaf_nodes {
                let current_node_voxel_lod =
                    ((node.aabb.max.x - node.aabb.min.x) / min_voxel_size).log2() as LodType;
                let current_voxel_size =
                    min_voxel_size * 2.0f32.powi(current_node_voxel_lod as i32);
                let current_depth = new_chunk_depth - current_node_voxel_lod;
                let current_chunk_voxel_num = new_chunk_voxel_num >> current_node_voxel_lod;
                let current_shape = RuntimeShape::<u32, 3>::new([
                    current_chunk_voxel_num,
                    current_chunk_voxel_num,
                    current_chunk_voxel_num,
                ]);

                trace!("current node voxel lod: {}, current_voxel size:{}, current depth: {}, current chunk shape size:{}", 
                        current_node_voxel_lod, current_voxel_size, current_depth, current_shape.size()
                );

                let mut new_node = node.clone();
                new_node.coord = (node.aabb.min - new_chunk_offset) / current_voxel_size;
                let index = current_shape.linearize(new_node.coord.as_uvec3().to_array());
                new_node.address = *node_address_mapper
                    .get(&current_depth)
                    .unwrap()
                    .get(index as usize)
                    .unwrap();
                seam_leaf_nodes.insert(new_node.address, new_node);
            }
        };

    if octrees.this.is_some() {
        get_seam_leaf_nodes(SubNodeIndex::X0Y0Z0, octrees.this.unwrap());
    }
    for octree in octrees.right.iter() {
        get_seam_leaf_nodes(SubNodeIndex::X1Y0Z0, octree.clone());
    }
    for octree in octrees.top.iter() {
        get_seam_leaf_nodes(SubNodeIndex::X0Y1Z0, octree.clone());
    }
    for octree in octrees.front.iter() {
        get_seam_leaf_nodes(SubNodeIndex::X0Y0Z1, octree.clone());
    }
    for octree in octrees.x_axis_edge.iter() {
        get_seam_leaf_nodes(SubNodeIndex::X0Y1Z1, octree.clone());
    }
    for octree in octrees.y_axis_edge.iter() {
        get_seam_leaf_nodes(SubNodeIndex::X1Y0Z1, octree.clone());
    }
    for octree in octrees.z_axis_edge.iter() {
        get_seam_leaf_nodes(SubNodeIndex::X1Y1Z0, octree.clone());
    }
    if octrees.vertex.is_some() {
        get_seam_leaf_nodes(SubNodeIndex::X1Y1Z1, octrees.vertex.unwrap());
    }

    let mut octree = Octree::new(shape);
    info!("seam_leaf_nodes size: {}", seam_leaf_nodes.len());
    // debug!("seam_leaf_nodes {:?}", seam_leaf_nodes);

    octree.address_node_map = Arc::new(RwLock::new(seam_leaf_nodes));
    Octree::build_bottom_up_from_leaf_nodes(
        &mut octree,
        min_voxel_size,
        new_chunk_offset,
        node_address_mapper,
    );
    info!(
        "build after seam_leaf_nodes size: {}",
        octree.address_node_map.read().unwrap().len()
    );

    check_octree_nodes_relation!(octree.address_node_map.clone());

    (entity, octree)
}

struct SeamNeighborOctrees {
    pub this: Option<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub this_lod: LodType,
    pub right: Vec<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub right_lod: LodType,
    pub top: Vec<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub top_lod: LodType,
    pub front: Vec<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub front_lod: LodType,
    pub x_axis_edge: Vec<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub x_axis_edge_lod: LodType,
    pub y_axis_edge: Vec<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub y_axis_edge_lod: LodType,
    pub z_axis_edge: Vec<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub z_axis_edge_lod: LodType,
    pub vertex: Option<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
    pub vertex_lod: LodType,
}

impl SeamNeighborOctrees {
    pub fn get_num(&self) -> usize {
        let mut num = 0;
        for i in self.right.iter() {
            num += if i.read().unwrap().len() > 0 { 1 } else { 0 };
        }
        for i in self.top.iter() {
            num += if i.read().unwrap().len() > 0 { 1 } else { 0 };
        }
        for i in self.front.iter() {
            num += if i.read().unwrap().len() > 0 { 1 } else { 0 };
        }
        for i in self.x_axis_edge.iter() {
            num += if i.read().unwrap().len() > 0 { 1 } else { 0 };
        }
        for i in self.y_axis_edge.iter() {
            num += if i.read().unwrap().len() > 0 { 1 } else { 0 };
        }
        for i in self.z_axis_edge.iter() {
            num += if i.read().unwrap().len() > 0 { 1 } else { 0 };
        }
        if self.vertex.is_some() {
            num += 1;
        }
        if self.this.is_some() {
            num += 1;
        }
        num
    }

    pub fn log(&self) {
        if self.this.is_some() {
            info!(
                "this num: {}, lod: {}",
                self.this.clone().unwrap().read().unwrap().len(),
                self.this_lod
            );
        }
        for (i, right) in self.right.iter().enumerate() {
            info!(
                "right[{}] num: {}, lod: {}",
                i,
                right.read().unwrap().len(),
                self.right_lod
            );
        }
        for (i, top) in self.top.iter().enumerate() {
            info!(
                "top[{}] num: {}, lod: {}",
                i,
                top.read().unwrap().len(),
                self.top_lod
            );
        }
        for (i, front) in self.front.iter().enumerate() {
            info!(
                "front[{}] num: {}, lod: {}",
                i,
                front.read().unwrap().len(),
                self.front_lod
            );
        }
        for (i, x_axis_edge) in self.x_axis_edge.iter().enumerate() {
            info!(
                "x_axis_edge[{}] num: {}, lod: {}",
                i,
                x_axis_edge.read().unwrap().len(),
                self.x_axis_edge_lod
            );
        }
        for (i, y_axis_edge) in self.y_axis_edge.iter().enumerate() {
            info!(
                "y_axis_edge[{}] num: {}, lod: {}",
                i,
                y_axis_edge.read().unwrap().len(),
                self.y_axis_edge_lod
            );
        }
        for (i, z_axis_edge) in self.z_axis_edge.iter().enumerate() {
            info!(
                "z_axis_edge[{}] num: {}, lod: {}",
                i,
                z_axis_edge.read().unwrap().len(),
                self.z_axis_edge_lod
            );
        }
        if self.vertex.is_some() {
            info!(
                "vertex num: {}, lod: {}",
                self.vertex.clone().unwrap().read().unwrap().len(),
                self.vertex_lod
            );
        }
    }

    pub fn get_min_lod(&self) -> LodType {
        self.this_lod
            .min(self.right_lod)
            .min(self.top_lod)
            .min(self.front_lod)
            .min(self.x_axis_edge_lod)
            .min(self.y_axis_edge_lod)
            .min(self.z_axis_edge_lod)
            .min(self.vertex_lod)
    }
}

#[derive(Debug)]
struct SeamNeighborAddresses {
    pub this: NodeAddress,
    pub right: Vec<NodeAddress>,
    pub top: Vec<NodeAddress>,
    pub front: Vec<NodeAddress>,
    pub x_axis_edge: Vec<NodeAddress>,
    pub y_axis_edge: Vec<NodeAddress>,
    pub z_axis_edge: Vec<NodeAddress>,
    pub vertex: NodeAddress,
}

// 找到相邻的chunk，获取所有的边界node，然后进行octree的构建
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn construct_octree(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, Octree)>,
    chunk_query: Query<
        (
            &TerrainChunkAddress,
            &TerrainChunkAabb,
            &TerrainChunkLod,
            &Children,
        ),
        With<TerrainChunk>,
    >,
    chunk_generator_query: Query<(&Octree, &TerrainChunkMainGenerator)>,
    mut seam_generator_query: ParamSet<(
        Query<(
            Entity,
            &Parent,
            &mut SeamMeshState,
            &mut TerrainChunkSeamGenerator,
        )>,
        Query<&mut SeamMeshState, With<TerrainChunkSeamGenerator>>,
    )>,
    setting: Res<TerrainSetting>,
    node_mapper: Res<OctreeDepthCoordMapper>,
    chunk_mapper: Res<TerrainChunkMapper>,
    lod_octree_node_query: Query<&LodOctreeNode>,
    lod_octree_map: Res<LodOctreeMap>,
    loader: Res<TerrainChunkLoader>,
) {
    if task_pool.is_idle() {
        for (entity, parent, mut state, mut _seam_generator) in seam_generator_query.p0().iter_mut()
        {
            if *state == SeamMeshState::ConstructOctree {
                let Ok((chunk_address, chunk_aabb, lod, _)) = chunk_query.get(parent.get()) else {
                    panic!("parent chunk musts exist!");
                };

                let _span =
                    info_span!("seam mesh construct octree", chunk_address = ?*chunk_address, lod = lod.get_lod())
                        .entered();

                let right_addresses = get_face_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    FaceIndex::Right,
                );
                let top_addresses = get_face_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    FaceIndex::Top,
                );
                let front_addresses = get_face_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    FaceIndex::Front,
                );
                let x_axis_edge_addresses = get_edge_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    EdgeIndex::XAxisY1Z1,
                );
                let y_axis_edge_addresses = get_edge_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    EdgeIndex::YAxisX1Z1,
                );
                let z_axis_edge_addresses = get_edge_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    EdgeIndex::ZAxisX1Y1,
                );
                let vertex_address = get_vertex_neighbor_lod_octree_nodes(
                    &lod_octree_node_query,
                    &lod_octree_map,
                    **chunk_address,
                    VertexIndex::X1Y1Z1,
                );
                let neighbor_addresses = SeamNeighborAddresses {
                    this: **chunk_address,
                    right: right_addresses,
                    top: top_addresses,
                    front: front_addresses,
                    x_axis_edge: x_axis_edge_addresses,
                    y_axis_edge: y_axis_edge_addresses,
                    z_axis_edge: z_axis_edge_addresses,
                    vertex: vertex_address,
                };

                info!("neighbor_addresses: {:#?}", neighbor_addresses);

                let get_octree = |address: &NodeAddress| -> (
                    Option<Arc<RwLock<HashMap<NodeAddress, Node>>>>,
                    LodType,
                ) {
                    let Some(entity) = chunk_mapper.get_chunk_entity(address.into()) else {
                        return (None, LodType::MAX);
                    };
                    if let Ok((chunk_address, _, lod, children)) = chunk_query.get(*entity) {
                        if loader.is_pending_unload(chunk_address) {
                            return (None, LodType::MAX);
                        }
                        for child in children.iter() {
                            if let Ok((octree, generator)) = chunk_generator_query.get(*child) {
                                assert!(generator.lod == lod.get_lod());
                                if octree.address_node_map.read().unwrap().len() > 0 {
                                    return (
                                        Some(octree.address_node_map.clone()),
                                        lod.get_lod(),
                                    );
                                }
                            }
                        }
                    }
                    (None, LodType::MAX)
                };
                let get_octrees = |addresses: &Vec<NodeAddress>| {
                    let mut min_lod = LodType::MAX;
                    let mut max_lod = LodType::MIN;
                    let mut octrees = Vec::with_capacity(addresses.len());
                    for address in addresses.iter() {
                        if let (Some(octree), lod) = get_octree(address) {
                            octrees.push(octree);
                            min_lod = min_lod.min(lod);
                            max_lod = max_lod.max(lod);
                        }
                    }
                    (octrees, min_lod, max_lod)
                };

                let this = get_octree(&neighbor_addresses.this);
                let right = get_octrees(&neighbor_addresses.right);
                let top = get_octrees(&neighbor_addresses.top);
                let front = get_octrees(&neighbor_addresses.front);
                let x_axis_edge = get_octrees(&neighbor_addresses.x_axis_edge);
                let y_axis_edge = get_octrees(&neighbor_addresses.y_axis_edge);
                let z_axis_edge = get_octrees(&neighbor_addresses.z_axis_edge);
                let vertex = get_octree(&neighbor_addresses.vertex);
                let octrees = SeamNeighborOctrees {
                    this: this.0,
                    this_lod: this.1,
                    right: right.0,
                    right_lod: right.1,
                    top: top.0,
                    top_lod: top.1,
                    front: front.0,
                    front_lod: front.1,
                    x_axis_edge: x_axis_edge.0,
                    x_axis_edge_lod: x_axis_edge.1,
                    y_axis_edge: y_axis_edge.0,
                    y_axis_edge_lod: y_axis_edge.1,
                    z_axis_edge: z_axis_edge.0,
                    z_axis_edge_lod: z_axis_edge.1,
                    vertex: vertex.0,
                    vertex_lod: vertex.1,
                };

                let max_lod = this
                    .1
                    .max(right.2)
                    .max(top.2)
                    .max(front.2)
                    .max(x_axis_edge.2)
                    .max(y_axis_edge.2)
                    .max(z_axis_edge.2)
                    .max(vertex.1);

                let num = octrees.get_num();
                info!(
                    "seam mesh construct octree: octree num:{}, chunk_address: {:?}",
                    num, chunk_address
                );

                octrees.log();

                if num < 2 {
                    *state = SeamMeshState::Done;
                    info!("seam mesh construct octree: {:?} fail", chunk_address);
                    continue;
                }

                // TODO: 加载过快，导致需要删除的mesh没有删除，找到了错误的mesh，导致lod误差变大。
                // TODO: 有待定加载的chunk，但是还没有加载，因此找到了更高lod的chunk，导致出现问题。
                let min_lod = octrees.get_min_lod();
                info!(
                    "min lod: {}, max_lod: {}, diff max: {}",
                    min_lod,
                    max_lod,
                    max_lod - min_lod
                );

                // TODO: assert change to debug assert
                assert!(max_lod - min_lod <= 3 || max_lod - min_lod >= 240);

                let min_voxel_size = setting.chunk_setting.get_voxel_size(min_lod);
                let mapper = node_mapper.mapper.clone();
                task_pool.spawn(construct_octree_task(
                    entity,
                    octrees,
                    mapper.clone(),
                    min_lod,
                    min_voxel_size,
                    *chunk_address,
                    chunk_aabb.clone(),
                ));
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished((entity, octree)) => {
                if let Some(mut entity_cmds) = commands.get_entity(entity) {
                    entity_cmds.insert(octree);
                    let mut query_1 = seam_generator_query.p1();
                    let mut state = query_1.get_mut(entity).expect(
                        "seam mesh state must be exist, because commands.get_entity is Some.",
                    );
                    *state = SeamMeshState::DualContouring;
                }
            }
        }
    }
}

async fn dual_contouring_run_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    node_addresses: Arc<RwLock<HashMap<NodeAddress, Node>>>,
    chunk_address: TerrainChunkAddress,
    lod: LodType,
) -> (Entity, MeshInfo) {
    let _span = info_span!("seam mesh dual contouring", ?chunk_address, lod).entered();

    let surface_guard: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();

    let mut mesh_info = MeshInfo::default();

    let mut default_visiter = DefaultDualContouringVisiter::new(surface_guard);
    let surface: std::sync::RwLockReadGuard<ShapeSurface> = surface.read().unwrap();
    let octree = OctreeProxy {
        node_addresses: node_addresses.read().unwrap(),
        is_seam: true,
        surface,
    };
    dual_contouring::dual_contouring(&octree, &mut default_visiter);

    info!("seam mesh positions: {}", default_visiter.positions.len());
    info!("seam mesh indices: {}", default_visiter.tri_indices.len());
    mesh_info.positions = default_visiter.positions;
    mesh_info.normals = default_visiter
        .normals
        .iter()
        .map(|n| (*n).into())
        .collect();
    mesh_info.indices = default_visiter.tri_indices;
    (entity, mesh_info)
}

#[allow(clippy::type_complexity)]
pub(crate) fn dual_contouring(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, MeshInfo)>,
    chunk_query: Query<(&TerrainChunkAddress, &TerrainChunkLod), With<TerrainChunk>>,
    mut chunk_generator_query: ParamSet<(
        Query<(Entity, &Parent, &Octree, &SeamMeshState), With<TerrainChunkSeamGenerator>>,
        Query<&mut SeamMeshState, With<TerrainChunkSeamGenerator>>,
    )>,
    surface: Res<IsosurfaceContext>,
) {
    if task_pool.is_idle() {
        for (entity, parent, octree, state) in chunk_generator_query.p0().iter() {
            if state == &SeamMeshState::DualContouring {
                let shape_surface = surface.shape_surface.clone();
                let octree_node_address = octree.address_node_map.clone();

                let (chunk_coord, chunk_lod) = chunk_query.get(parent.get()).unwrap();
                task_pool.spawn(dual_contouring_run_task(
                    entity,
                    shape_surface,
                    octree_node_address,
                    *chunk_coord,
                    chunk_lod.get_lod(),
                ));
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished((entity, mesh_info)) => {
                if let Some(mut entity_cmds) = commands.get_entity(entity) {
                    entity_cmds.insert(mesh_info);
                    if let Ok(mut state) = chunk_generator_query.p1().get_mut(entity) {
                        *state = SeamMeshState::CreateMesh;
                    }
                }
            }
        }
    }
}
