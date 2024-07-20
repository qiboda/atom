use std::sync::{Arc, RwLock};

use bevy::{math::Vec3A, prelude::*, utils::hashbrown::HashMap};
use ndshape::RuntimeShape;
use strum::IntoEnumIterator;

use crate::{
    chunk_mgr::{
        chunk::{
            bundle::TerrainChunk,
            chunk_lod::{LodType, TerrainChunkLod},
        },
        chunk_mapper::TerrainChunkMapper,
    },
    isosurface::{
        comp::{SeamMeshState, TerrainChunkMainGenerator, TerrainChunkSeamGenerator},
        dc::octree::{check_octree_nodes_relation, node::NodeType, tables::SubNodeIndex},
        mesh::mesh_info::MeshInfo,
        surface::shape_surface::{IsosurfaceContext, ShapeSurface},
    },
    setting::TerrainSetting,
};

use super::{
    dual_contouring::{self, DefaultDualContouringVisiter},
    octree::{address::NodeAddress, node::Node, Octree, OctreeProxy},
    OctreeDepthCoordMapper,
};
use bevy_async_task::AsyncTaskPool;
use terrain_core::chunk::coords::TerrainChunkCoord;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
async fn construct_octree_task(
    entity: Entity,
    octrees: [Option<Arc<RwLock<HashMap<NodeAddress, Node>>>>; 8],
    node_address_mapper: Arc<RwLock<HashMap<u16, Vec<NodeAddress>>>>,
    lod: LodType,
    chunk_size: f32,
    voxel_size: f32,
    chunk_coord: TerrainChunkCoord,
) -> (Entity, Octree) {
    let _span = debug_span!("seam mesh construct octree task", %chunk_coord, lod).entered();

    let mut seam_leaf_nodes: HashMap<NodeAddress, Node> = HashMap::default();
    octrees.iter().enumerate().for_each(|(i, octree)| {
        let Some(octree) = octree else {
            return;
        };

        let node_map = octree.read().unwrap();
        if node_map.len() < 1 {
            return;
        }

        let octree_aabb = node_map.get(&NodeAddress::root()).unwrap().aabb;
        let subnode_index = SubNodeIndex::from_repr(i).unwrap();
        let leaf_nodes = Octree::get_all_seam_leaf_nodes(&node_map, octree_aabb, subnode_index);
        debug!("{}th, get leaf nodes: {}", i, leaf_nodes.len());

        let index_array = subnode_index.to_array();
        let parent_address = NodeAddress::root().get_child_address(subnode_index);
        for node in leaf_nodes {
            let mut new_node = node.clone();
            new_node.address = parent_address.concat_address(new_node.address);
            new_node.coord += Vec3A::new(
                index_array[0] as f32,
                index_array[1] as f32,
                index_array[2] as f32,
            ) * 16.0;
            seam_leaf_nodes.insert(new_node.address, new_node);
        }
    });

    let lod_voxel_size = voxel_size * 2.0_f32.powf(lod as f32);
    let offset = chunk_coord * chunk_size;
    // 两倍的chunk size，因为是相邻chunk的边界
    let size = (chunk_size * 2.0 / lod_voxel_size) as u32;
    let shape = RuntimeShape::<u32, 3>::new([size, size, size]);
    debug!("lod_voxel size: {}, size: {}", lod_voxel_size, size);

    let mut octree = Octree::new(shape);
    info!("seam_leaf_nodes size: {}", seam_leaf_nodes.len());
    debug!("seam_leaf_nodes {:?}", seam_leaf_nodes);

    let addresses = seam_leaf_nodes.clone();

    octree.address_node_map = Arc::new(RwLock::new(seam_leaf_nodes));
    Octree::build_bottom_up_from_leaf_nodes(
        &mut octree,
        lod_voxel_size,
        offset,
        node_address_mapper,
    );
    info!(
        "build after seam_leaf_nodes size: {}",
        octree.address_node_map.read().unwrap().len()
    );

    octree
        .address_node_map
        .read()
        .unwrap()
        .iter()
        .for_each(|(address, node)| {
            if node.node_type == NodeType::Leaf {
                let old_node = addresses.get(address).unwrap();
                assert!(old_node.address == node.address);
                assert!(old_node.aabb.min == node.aabb.min);
                assert!(old_node.aabb.max == node.aabb.max);
                assert!(old_node.vertices_mat_types == node.vertices_mat_types);
                assert!(old_node.vertex_estimate == node.vertex_estimate);
            }
        });

    check_octree_nodes_relation!(octree.address_node_map.clone());

    (entity, octree)
}

// 找到相邻的chunk，获取所有的边界node，然后进行octree的构建
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn construct_octree(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, Octree)>,
    chunk_query: Query<(&TerrainChunkCoord, &TerrainChunkLod, &Children), With<TerrainChunk>>,
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
) {
    if task_pool.is_idle() {
        for (entity, parent, mut state, mut seam_generator) in seam_generator_query.p0().iter_mut()
        {
            if *state == SeamMeshState::ConstructOctree {
                let Ok((chunk_coord, lod, _)) = chunk_query.get(parent.get()) else {
                    panic!("parent chunk musts exist!");
                };

                let _span =
                    info_span!("seam mesh construct octree", chunk_coord = %*chunk_coord, lod = lod.get_lod())
                        .entered();

                let mut chunk_entities = [None, None, None, None, None, None, None, None];
                for vi in SubNodeIndex::iter() {
                    let offset = TerrainChunkCoord::from(vi.to_array());
                    let chunk_entity =
                        chunk_mapper.get_chunk_entity_by_coord(chunk_coord + &offset);
                    chunk_entities[vi as usize] = chunk_entity;
                    info!(
                        "seam mesh construct octree: coord: {} add chunk entity, offset: {}",
                        chunk_coord, offset
                    );
                }

                // 获取八个chunk的最小lod和nodes
                let mut min_lod = LodType::MAX;
                let mut octrees = [None, None, None, None, None, None, None, None];
                for (index, chunk_entity) in chunk_entities.iter().enumerate() {
                    if let Some(chunk_entity) = chunk_entity {
                        if let Ok((_, lod, children)) = chunk_query.get(**chunk_entity) {
                            for child in children.iter() {
                                if let Ok((octree, generator)) = chunk_generator_query.get(*child) {
                                    if generator.lod == lod.get_lod() {
                                        if lod.get_lod() < min_lod {
                                            min_lod = lod.get_lod();
                                        }
                                        octrees[index] = Some(octree.address_node_map.clone());
                                        let offset =
                                            SubNodeIndex::from_repr(index).unwrap().to_array();
                                        seam_generator
                                            .lod_map
                                            .insert(chunk_coord + &offset.into(), lod.get_lod());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                let mut num = 0;
                for octree in octrees.iter().flatten() {
                    let node_map = octree.read().unwrap();
                    if node_map.len() > 0 {
                        num += 1;
                    }
                }

                info!(
                    "seam mesh construct octree: octree num:{}, num: {}, chunk_coord: {}",
                    octrees.len(),
                    num,
                    chunk_coord
                );

                if num < 2 {
                    *state = SeamMeshState::Done;
                    info!("seam mesh construct octree: {} fail", chunk_coord);
                    continue;
                }
                // TODO: assert change to debug assert
                debug_assert_ne!(min_lod, LodType::MAX);

                let chunk_size = setting.chunk_setting.chunk_size;
                let voxel_size = setting.chunk_setting.voxel_size;
                let mapper = node_mapper.mapper.clone();
                task_pool.spawn(construct_octree_task(
                    entity,
                    octrees,
                    mapper.clone(),
                    min_lod,
                    chunk_size,
                    voxel_size,
                    *chunk_coord,
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
    chunk_coord: TerrainChunkCoord,
    lod: LodType,
) -> (Entity, MeshInfo) {
    let _span = debug_span!("seam mesh dual contouring", %chunk_coord, lod).entered();

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
    chunk_query: Query<(&TerrainChunkCoord, &TerrainChunkLod), With<TerrainChunk>>,
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
