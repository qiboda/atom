use bevy::{
    math::{bounding::BoundingVolume, Vec3A},
    prelude::*,
    utils::HashMap,
};

use crate::{
    chunk_mgr::chunk::{
        bundle::TerrainChunkBundle,
        chunk_aabb::TerrainChunkAabb,
        state::{TerrainChunkAddress, TerrainChunkSeamLod, TerrainChunkState},
    },
    lod::{
        lod_octree::{TerrainLodOctree, TerrainLodOctreeNode},
        neighbor_query::{
            get_edge_neighbor_lod_octree_nodes, get_face_neighbor_lod_octree_nodes,
            get_vertex_neighbor_lod_octree_nodes,
        },
    },
    tables::{EdgeIndex, FaceIndex, SubNodeIndex, VertexIndex},
};

use super::{
    chunk::bundle::TerrainChunk,
    chunk_loader::{TerrainChunkLoadEvent, TerrainChunkReloadEvent, TerrainChunkUnLoadEvent},
};

#[derive(Debug, Resource, Default)]
pub struct TerrainChunkMapper {
    /// entity is TerrainChunk
    pub data: HashMap<TerrainChunkAddress, Entity>,
}

impl TerrainChunkMapper {
    pub fn get_chunk_entity(&self, terrain_chunk_address: TerrainChunkAddress) -> Option<&Entity> {
        self.data.get(&terrain_chunk_address)
    }

    pub fn new() -> TerrainChunkMapper {
        Self::default()
    }
}

pub fn trigger_chunk_reload_event(
    event_trigger: Trigger<TerrainChunkReloadEvent>,
    terrain_chunk_mapper: Res<TerrainChunkMapper>,
    mut query: Query<(Entity, &mut TerrainChunkState), With<TerrainChunk>>,
) {
    for node_address in event_trigger.event().node_addresses.iter() {
        let terrain_chunk_address: TerrainChunkAddress = node_address.into();
        let Some(entity) = terrain_chunk_mapper.data.get(&terrain_chunk_address) else {
            return;
        };
        if let Ok((_chunk_entity, mut state)) = query.get_mut(*entity) {
            // *state = TerrainChunkState::CREATE_SEAM_MESH;
            // commands.trigger(TerrainChunkCreateMainMeshEvent {
            //     entity: chunk_entity,
            //     lod: chunk_lod.get_lod(),
            // });
        }
    }
}

pub fn trigger_chunk_load_event(
    event_trigger: Trigger<TerrainChunkLoadEvent>,
    mut commands: Commands,
    mut terrain_chunk_mapper: ResMut<TerrainChunkMapper>,
    lod_octree: Res<TerrainLodOctree>,
    mut query: Query<(&mut TerrainChunkState, &mut TerrainChunkSeamLod), With<TerrainChunk>>,
) {
    for node_address in event_trigger.event().node_addresses.iter() {
        let current_node = lod_octree.get_node(node_address).unwrap();

        let mut set_seam_state = |x: Vec<&TerrainLodOctreeNode>| {
            for node in x {
                if let Some(chunk_entity) = terrain_chunk_mapper.get_chunk_entity(node.code.into())
                {
                    if let Ok((mut chunk_state, mut seam_lod)) = query.get_mut(*chunk_entity) {
                        *chunk_state |= TerrainChunkState::CREATE_SEAM_MESH;
                        let lod = get_node_seam_lod(node, &lod_octree);
                        seam_lod.0 = lod;
                    }
                }
            }
        };

        let left_face_nodes =
            get_face_neighbor_lod_octree_nodes(&lod_octree, current_node, FaceIndex::Left, 1);
        for node in left_face_nodes.iter() {
            debug!(
                "left node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(left_face_nodes);
        let back_face_nodes =
            get_face_neighbor_lod_octree_nodes(&lod_octree, current_node, FaceIndex::Back, 1);
        for node in back_face_nodes.iter() {
            debug!(
                "back node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(back_face_nodes);
        let bottom_face_nodes =
            get_face_neighbor_lod_octree_nodes(&lod_octree, current_node, FaceIndex::Bottom, 1);
        for node in bottom_face_nodes.iter() {
            debug!(
                "bottom node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(bottom_face_nodes);

        let x00_axis_edge_nodes =
            get_edge_neighbor_lod_octree_nodes(&lod_octree, current_node, EdgeIndex::XAxisY0Z0, 1);
        for node in x00_axis_edge_nodes.iter() {
            debug!(
                "x00 edge node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(x00_axis_edge_nodes);
        let y00_axis_edge_nodes =
            get_edge_neighbor_lod_octree_nodes(&lod_octree, current_node, EdgeIndex::YAxisX0Z0, 1);
        for node in y00_axis_edge_nodes.iter() {
            debug!(
                "y00 edge node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(y00_axis_edge_nodes);
        let z00_axis_edge_nodes =
            get_edge_neighbor_lod_octree_nodes(&lod_octree, current_node, EdgeIndex::ZAxisX0Y0, 1);
        for node in z00_axis_edge_nodes.iter() {
            debug!(
                "z00 edge node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(z00_axis_edge_nodes);

        let vertex_000_nodes =
            get_vertex_neighbor_lod_octree_nodes(&lod_octree, current_node, VertexIndex::X0Y0Z0, 1);
        for node in vertex_000_nodes.iter() {
            debug!(
                "000 vertex node: address: {:?}, center: {:?}",
                node.code,
                node.aabb.center()
            );
        }
        set_seam_state(vertex_000_nodes);

        let chunk_address = TerrainChunkAddress::from(*node_address);
        // if terrain_chunk_mapper
        //     .get_chunk_entity(chunk_address)
        //     .is_some()
        // {
        //     continue;
        // }

        // TerrainChunkState::CREATE_MAIN_MESH |
        let mut bundle = TerrainChunkBundle::new(
            TerrainChunkState::CREATE_SEAM_MESH | TerrainChunkState::CREATE_MAIN_MESH,
        );
        bundle.terrain_chunk_address = chunk_address;
        bundle.terrain_chunk_aabb = TerrainChunkAabb(current_node.aabb);

        debug!(
            "spawn_terrain_chunks: {:?}, address: {:?}",
            bundle.terrain_chunk_aabb.min, bundle.terrain_chunk_address,
        );

        let lod = get_node_seam_lod(current_node, &lod_octree);
        bundle.terrain_chunk_seam_lod = TerrainChunkSeamLod(lod);
        let chunk_entity = commands.spawn(bundle).id();
        let value = terrain_chunk_mapper
            .data
            .insert(chunk_address, chunk_entity);
        assert!(value.is_none());
    }
}

fn get_node_seam_lod(
    current_node: &TerrainLodOctreeNode,
    lod_octree: &Res<TerrainLodOctree>,
) -> [[u8; 8]; 8] {
    // lod
    let set_lod = |x: &Vec<&TerrainLodOctreeNode>,
                   lod: &mut [[u8; 8]; 8],
                   min_location: Vec3A,
                   half_size: Vec3A,
                   index: SubNodeIndex| {
        if x.is_empty() {
            lod[index.to_index()][0] = lod[0][0];
            lod[index.to_index()][1] = lod[0][1];
            lod[index.to_index()][2] = lod[0][2];
            lod[index.to_index()][3] = lod[0][3];
            lod[index.to_index()][4] = lod[0][4];
            lod[index.to_index()][5] = lod[0][5];
            lod[index.to_index()][6] = lod[0][6];
            lod[index.to_index()][7] = lod[0][7];
        } else {
            lod[index.to_index()][0] = x[0].code.level();
            lod[index.to_index()][1] = x[0].code.level();
            lod[index.to_index()][2] = x[0].code.level();
            lod[index.to_index()][3] = x[0].code.level();
            lod[index.to_index()][4] = x[0].code.level();
            lod[index.to_index()][5] = x[0].code.level();
            lod[index.to_index()][6] = x[0].code.level();
            lod[index.to_index()][7] = x[0].code.level();
            if x.len() > 1 {
                let offset = index.to_array().map(|x| x as f32);
                let offset = Vec3A::new(offset[0], offset[1], offset[2]);
                for node in x.iter() {
                    let sub_index = ((node.aabb.min - min_location - offset * half_size * 2.0)
                        / half_size)
                        .round();
                    debug!("sub_index: {}, node.aabb.min: {}", sub_index, node.aabb.min);
                    let sub_index = (sub_index.z * 4.0 + sub_index.y * 2.0 + sub_index.x) as usize;
                    debug!(
                        "sub_index: {}, min_location: {}, half_size: {}, offset: {}",
                        sub_index, min_location, half_size, offset
                    );
                    if sub_index < 8 {
                        lod[index.to_index()][sub_index] = node.code.level();
                    }
                }
            }
        }
    };
    // 每个sub node的两个lod
    let mut lod = [[0; 8]; 8];
    lod[0][0] = current_node.code.level();
    lod[0][1] = current_node.code.level();
    lod[0][2] = current_node.code.level();
    lod[0][3] = current_node.code.level();
    lod[0][4] = current_node.code.level();
    lod[0][5] = current_node.code.level();
    lod[0][6] = current_node.code.level();
    lod[0][7] = current_node.code.level();

    let min_location = current_node.aabb.min;
    let half_size = current_node.aabb.half_size();

    let depth = 3;
    let right_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Right, depth);
    for node in right_face_nodes.iter() {
        debug!(
            "set lod right node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }
    set_lod(
        &right_face_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X1Y0Z0,
    );
    let front_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Front, depth);
    for node in front_face_nodes.iter() {
        debug!(
            "set lod front node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }
    set_lod(
        &front_face_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X0Y0Z1,
    );
    let top_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Top, depth);
    for node in top_face_nodes.iter() {
        debug!(
            "set lod top node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }
    set_lod(
        &top_face_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X0Y1Z0,
    );

    let x11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY1Z1, depth);
    set_lod(
        &x11_axis_edge_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X0Y1Z1,
    );
    for node in x11_axis_edge_nodes.iter() {
        debug!(
            "set lod x11 edge node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }
    let y11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX1Z1, depth);
    set_lod(
        &y11_axis_edge_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X1Y0Z1,
    );
    for node in y11_axis_edge_nodes.iter() {
        debug!(
            "set lod y11 edge node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }
    let z11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX1Y1, depth);
    set_lod(
        &z11_axis_edge_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X1Y1Z0,
    );
    for node in z11_axis_edge_nodes.iter() {
        debug!(
            "set lod z11 edge node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }

    let vertex_111_nodes =
        get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X1Y1Z1, depth);
    set_lod(
        &vertex_111_nodes,
        &mut lod,
        min_location,
        half_size,
        SubNodeIndex::X1Y1Z1,
    );
    for node in vertex_111_nodes.iter() {
        debug!(
            "set lod vertex 111 node: address: {:?}, center: {:?}",
            node.code,
            node.aabb.center()
        );
    }

    // 最深的node。也就是voxel size最小
    let max_value = *lod.iter().flatten().max().unwrap();
    debug!(
        "set lod current lod: {:?} address: {:?}, center: {:?}",
        lod,
        current_node,
        current_node.aabb.center()
    );
    // 当前节点需要增加的深度的值。
    lod.iter_mut().flatten().for_each(|x| {
        *x = max_value - *x;
    });

    debug!(
        "set lod new lod: {:?} address: {:?}, center: {:?}",
        lod,
        current_node,
        current_node.aabb.center()
    );
    assert!(lod.iter().flatten().all(|x| *x <= 2));
    lod
}

pub fn trigger_chunk_unload_event(
    event_trigger: Trigger<TerrainChunkUnLoadEvent>,
    mut terrain_chunk_mapper: ResMut<TerrainChunkMapper>,
    mut commands: Commands,
) {
    for node_address in event_trigger.event().node_addresses.iter() {
        let terrain_chunk_address = node_address.into();
        if let Some(chunk_entity) = terrain_chunk_mapper.get_chunk_entity(terrain_chunk_address) {
            commands.get_entity(*chunk_entity).map(|x| {
                x.despawn_recursive();
                // info!(
                //     "trigger_chunk_unload_event despawn: {:?}",
                //     terrain_chunk_address
                terrain_chunk_mapper.data.remove(&terrain_chunk_address)
            });
        }
    }
}
