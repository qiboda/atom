use std::ops::Not;

use bevy::{prelude::*, utils::hashbrown::HashSet};

use crate::{
    isosurface::{
        comp::{
            MainMeshState, SeamMeshState, TerrainChunkCreateSeamMeshEvent,
            TerrainChunkMainGenerator, TerrainChunkSeamGenerator,
        },
        dc::octree::tables::{EdgeIndex, FaceIndex, VertexIndex},
    },
    lod::{
        lod_octree::{LodOctreeMap, LodOctreeNode},
        neighbor_query::{
            get_edge_neighbor_lod_octree_nodes, get_face_neighbor_lod_octree_nodes,
            get_vertex_neighbor_lod_octree_nodes,
        },
    },
};

use super::{
    chunk::{
        bundle::TerrainChunk,
        chunk_lod::TerrainChunkLod,
        state::{SeamMeshIdGenerator, TerrainChunkAddress, TerrainChunkState},
    },
    chunk_loader::TerrainChunkLoader,
    chunk_mapper::TerrainChunkMapper,
};

pub fn update_to_wait_create_seam(
    mut query: Query<
        (
            &Children,
            &mut TerrainChunkState,
            &TerrainChunkLod,
            &TerrainChunkAddress,
        ),
        With<TerrainChunk>,
    >,
    mut generator_query: Query<(
        Option<&mut Visibility>,
        &MainMeshState,
        &TerrainChunkMainGenerator,
    )>,
) {
    for (children, mut chunk_state, chunk_lod, chunk_address) in query.iter_mut() {
        if TerrainChunkState::CreateMainMesh == *chunk_state {
            let mut count = 0;
            for child in children.iter() {
                if let Ok((visibility, mesh_state, generator)) = generator_query.get_mut(*child) {
                    if *mesh_state == MainMeshState::Done && generator.lod == chunk_lod.get_lod() {
                        info!(
                            "update_to_wait_create_seam:{:?}, lod: {}",
                            chunk_address,
                            chunk_lod.get_lod()
                        );
                        *chunk_state = TerrainChunkState::WaitToCreateSeam;
                        if let Some(mut visibility) = visibility {
                            *visibility = Visibility::Visible;
                        }
                        count += 1;
                    }
                }
            }
            assert!(count < 2);
        }
    }
}

// 检测是否所有的主mesh都创建了，如果都创建了，且有需要更新缝隙的，更新缝隙
#[allow(clippy::type_complexity)]
pub fn to_create_seam_mesh(
    mut query: ParamSet<(
        Query<&TerrainChunkState, With<TerrainChunk>>,
        Query<(&TerrainChunkAddress, &TerrainChunkState), With<TerrainChunk>>,
        Query<(&mut TerrainChunkState, &mut SeamMeshIdGenerator), With<TerrainChunk>>,
    )>,
    chunk_mapper: Res<TerrainChunkMapper>,
    mut event_writer: EventWriter<TerrainChunkCreateSeamMeshEvent>,
    lod_octree_node_query: Query<&LodOctreeNode>,
    lod_octree_map: Res<LodOctreeMap>,
) {
    let exist_create_main_mesh_state = query.p0().iter().any(|state| {
        if *state == TerrainChunkState::CreateMainMesh {
            // debug!("state is create main mesh, state: {:?}", state);
            return true;
        }
        false
    });

    if exist_create_main_mesh_state {
        return;
    }

    // 找到所有刚刚创建了MainMesh的chunk
    let mut to_create_seam_chunks = vec![];
    for (chunk_address, state) in query.p1().iter_inner() {
        if TerrainChunkState::WaitToCreateSeam == *state {
            to_create_seam_chunks.push(chunk_address);
        }
    }

    let mut update_seam_chunk_addresses = HashSet::new();
    to_create_seam_chunks.iter().for_each(|x| {
        let left_face_addresses = get_face_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            FaceIndex::Left,
        );
        let bottom_face_addresses = get_face_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            FaceIndex::Bottom,
        );
        let back_face_addresses = get_face_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            FaceIndex::Back,
        );
        let x_axis_edge_addresses = get_edge_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            EdgeIndex::XAxisY0Z0,
        );
        let y_axis_edge_addresses = get_edge_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            EdgeIndex::YAxisX0Z0,
        );
        let z_axis_edge_addresses = get_edge_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            EdgeIndex::ZAxisX0Y0,
        );
        let vertex_address = get_vertex_neighbor_lod_octree_nodes(
            &lod_octree_node_query,
            &lod_octree_map,
            ***x,
            VertexIndex::X0Y0Z0,
        );
        update_seam_chunk_addresses.insert(***x);
        update_seam_chunk_addresses.extend(left_face_addresses);
        update_seam_chunk_addresses.extend(bottom_face_addresses);
        update_seam_chunk_addresses.extend(back_face_addresses);
        update_seam_chunk_addresses.extend(x_axis_edge_addresses);
        update_seam_chunk_addresses.extend(y_axis_edge_addresses);
        update_seam_chunk_addresses.extend(z_axis_edge_addresses);
        update_seam_chunk_addresses.insert(vertex_address);
    });

    info!(
        "update_seam_chunk_address: {:#?}",
        update_seam_chunk_addresses
    );

    for chunk_address in update_seam_chunk_addresses {
        if let Some(entity) = chunk_mapper.get_chunk_entity(chunk_address.into()) {
            if let Ok((mut state, mut seam_mesh_id_generator)) = query.p2().get_mut(*entity) {
                let seam_mesh_id = seam_mesh_id_generator.gen();
                *state = TerrainChunkState::CreateSeamMesh;
                info!(
                    "to create seam chunks, address: {:?}, id:{:?}, current id: {:?}",
                    chunk_address,
                    seam_mesh_id,
                    seam_mesh_id_generator.current(),
                );
                event_writer.send(TerrainChunkCreateSeamMeshEvent {
                    chunk_entity: *entity,
                    seam_mesh_id,
                });
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_create_seam_mesh_over(
    mut query: Query<
        (
            &Children,
            &mut TerrainChunkState,
            &TerrainChunkAddress,
            &SeamMeshIdGenerator,
        ),
        With<TerrainChunk>,
    >,
    mut generator_query: Query<(&SeamMeshState, &TerrainChunkSeamGenerator)>,
) {
    for (children, mut chunk_state, chunk_address, id_generator) in query.iter_mut() {
        if TerrainChunkState::CreateSeamMesh == *chunk_state {
            let mut count = 0;
            for child in children {
                if let Ok((state, seam_generator)) = generator_query.get_mut(*child) {
                    if *state == SeamMeshState::Done {
                        count += 1;
                        if id_generator.current() == seam_generator.seam_mesh_id {
                            info!(
                                "update_create_seam_mesh_over: {:?}, {:?}",
                                chunk_address,
                                id_generator.current()
                            );
                            *chunk_state = TerrainChunkState::Done;
                        }
                    }
                }
            }
            assert!(count < 2);
        }
    }
}

pub fn hidden_main_mesh(
    query: Query<
        (
            &Children,
            &TerrainChunkLod,
            &TerrainChunkState,
            &TerrainChunkAddress,
        ),
        With<TerrainChunk>,
    >,
    chunk_query: Query<(), With<TerrainChunk>>,
    mut generator_query: Query<(&ViewVisibility, &mut Visibility, &TerrainChunkMainGenerator)>,
    chunk_mapper: Res<TerrainChunkMapper>,
    lod_octree_node_query: Query<&LodOctreeNode>,
    lod_octree_map: Res<LodOctreeMap>,
    loader: Res<TerrainChunkLoader>,
) {
    let all_create_over = query.iter().all(|(_, _, state, _)| {
        if *state == TerrainChunkState::Done {
            return true;
        }
        false
    });

    if all_create_over.not() {
        return;
    }
    info!(
        "all mesh create over, chunk_mapper len: {}, lod node num: {}, lod map len: {}, loaded leaf node set len: {}, pending unload leaf node set len: {}, leaf node pending load deque len: {}, chunk_num:{}",
        chunk_mapper.data.len(),
        lod_octree_node_query.iter().len(),
        lod_octree_map.node_map.len(),
        loader.loaded_leaf_node_set.len(),
        loader.pending_unload_leaf_node_set.len(),
        loader.leaf_node_pending_load_deque.len(),
        chunk_query.iter().len()
    );

    for (children, chunk_lod, _, chunk_address) in query.iter() {
        debug!("main_mesh_visibility: {:?}", chunk_address);
        for child in children.iter() {
            if let Ok((_, mut visibility, generator)) = generator_query.get_mut(*child) {
                if generator.lod != chunk_lod.get_lod() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
