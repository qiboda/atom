use std::ops::Not;

use bevy::{math::bounding::BoundingVolume, prelude::*};
use ndshape::RuntimeShape;
use strum::IntoEnumIterator;

use crate::{
    chunk_mgr::{
        chunk::comp::{
            TerrainChunkAabb, TerrainChunkAddress, TerrainChunkBorderVertices,
            TerrainChunkNeighborLodNodes, TerrainChunkSeamLod, TerrainChunkState,
        },
        chunk_mapper::TerrainChunkMapper,
    },
    isosurface::dc::{
        cpu_dc::{
            octree::{check_octree_nodes_relation, OctreeLevel},
            seam_connect::seam_connect,
        },
        gpu_dc::mesh_compute::{
            TerrainChunkMeshData, TerrainChunkMeshDataRenderWorldSender,
            TerrainChunkRenderBorderVertices, TerrainChunkSeamMeshData,
        },
    },
    lod::morton_code::MortonCode,
    setting::{StitchSeamScheme, TerrainSetting},
    tables::SubNodeIndex,
};

use super::{
    dual_contouring::{self, DefaultDualContouringVisiter},
    octree::{
        node::{Node, NodeType},
        Octree, OctreeProxy,
    },
};

fn get_seam_leaf_nodes(
    octree: &mut Octree,
    subnode_index: SubNodeIndex,
    border_vertices: &TerrainChunkBorderVertices,
    current_chunk_aabb: &TerrainChunkAabb,
    seam_voxel_size: f32,
    seam_chunk_depth: u8,
) {
    let leaf_nodes_index = Octree::get_all_seam_leaf_nodes_by_aabb(
        border_vertices,
        current_chunk_aabb.0,
        subnode_index,
    );

    for index in leaf_nodes_index {
        let voxel_vertex = border_vertices.vertices[index];
        let voxel_aabb = border_vertices.vertices_aabb[index];
        let voxel_size = voxel_aabb.half_size().x * 2.0;

        // TODO 统一改为depth，不再使用level
        let up_depth = (voxel_size / seam_voxel_size).log(2.0) as u8;
        let node_depth = seam_chunk_depth - up_depth;

        let node_coord = (voxel_aabb.min - current_chunk_aabb.min) / voxel_size;

        debug!(
            "voxel_vertex: {:?}, voxel_aabb: {:?}, voxel_size: {}, up_depth: {}, node_coord: {}, node_depth: {}",
            voxel_vertex, voxel_aabb, voxel_size, up_depth, node_coord, node_depth
        );

        let node_address = MortonCode::encode(node_coord.as_uvec3(), node_depth);
        let mut node = Node::new(NodeType::Leaf, node_address);
        node.vertex_estimate = voxel_vertex.vertex_location.xyz();
        node.normal_estimate = voxel_vertex.vertex_normal.xyz();
        node.vertices_side_types = voxel_vertex.get_voxel_side();
        node.vertices_biomes = voxel_vertex.get_voxel_biome();
        node.aabb = voxel_aabb;

        octree.insert_leaf_node(node);
    }
}

// 找到相邻的chunk，获取所有的边界node，然后进行octree的构建
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_seam_mesh(
    chunk_query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAddress,
        &TerrainChunkAabb,
        &TerrainChunkSeamLod,
        &TerrainChunkNeighborLodNodes,
    )>,
    terrain_chunk_mapper: Res<TerrainChunkMapper>,
    terrain_setting: Res<TerrainSetting>,
    sender: Res<TerrainChunkMeshDataRenderWorldSender>,
    render_border_vertices: Res<TerrainChunkRenderBorderVertices>,
) {
    chunk_query.par_iter().for_each(|(entity, state, address, current_chunk_aabb, seam_lod, neighbor_nodes)|{
        if state.contains(TerrainChunkState::CREATE_SEAM_MESH).not() {
            return;
        }

        let _span = info_span!("cpu dc create seam mesh one").entered();

        let seam_chunk_min = current_chunk_aabb.min;
        let add_lod = seam_lod.get_lod(SubNodeIndex::X0Y0Z0);
        let seam_chunk_lod_depth = address.0.depth() + add_lod[0];
        let seam_voxel_size = terrain_setting.get_voxel_size(seam_chunk_lod_depth);
        let seam_chunk_size =
            terrain_setting.get_chunk_size(seam_chunk_lod_depth - add_lod[0]) * 2.0;
        let seam_voxel_num = (seam_chunk_size / seam_voxel_size).round() as u32;
        let seam_chunk_depth = seam_voxel_num.ilog2() as u8;

        debug!("chunk_min: {:?}, add_lod: {}, seam_chunk_depth: {}, seam_voxel_size: {}, seam_chunk_size: {}, seam_voxel_num: {}",
            seam_chunk_min, add_lod[0], seam_chunk_lod_depth, seam_voxel_size, seam_chunk_size, seam_voxel_num);

        let shape = RuntimeShape::<u32, 3>::new([seam_voxel_num, seam_voxel_num, seam_voxel_num]);
        let mut octree = Octree::new(shape);
        octree.levels.resize_with(seam_chunk_depth as usize + 1, OctreeLevel::default);

        for subnode_index in SubNodeIndex::iter() {
            let nodes = &neighbor_nodes.nodes[subnode_index.to_index()];
            for node in nodes {
                if let Some(chunk_entity) =
                    terrain_chunk_mapper.get_chunk_entity(TerrainChunkAddress::new(node.code))
                {
                    if let Some(border_vertices) = render_border_vertices.map.get(chunk_entity) {
                        get_seam_leaf_nodes(
                            &mut octree,
                            subnode_index,
                            border_vertices,
                            current_chunk_aabb,
                            seam_voxel_size,
                            seam_chunk_depth,
                        );
                    }
                }
            }
        }

        let mesh = match terrain_setting.stitch_seam_scheme {
            StitchSeamScheme::DualContouring => {
                debug!(
                    "chunk_min: {}, voxel num: {}, seam_leaf_nodes size: {} before octree build",
                    seam_chunk_min, seam_voxel_num, octree.get_nodes_num()
                );

                Octree::build_bottom_up_from_leaf_nodes(&mut octree, seam_voxel_size);
                check_octree_nodes_relation!(&octree);
                debug!(
                    "chunk_min: {}, voxel num: {}, seam_leaf_nodes size: {} after octree build",
                    seam_chunk_min, seam_voxel_num, octree.get_nodes_num()
                );

                let mut default_visiter = DefaultDualContouringVisiter::default();
                let octree = OctreeProxy {
                    octree: &octree,
                    is_seam: true,
                    chunk_min: seam_chunk_min,
                };
                dual_contouring::dual_contouring(&octree, &mut default_visiter);

                debug!(
                    "chunk_min: {}, seam mesh positions: {}, positions: {:?}",
                    seam_chunk_min,
                    default_visiter.positions.len(),
                    default_visiter.positions
                );
                debug!(
                    "chunk_min: {}, seam mesh indices: {}, indices: {:?}",
                    seam_chunk_min,
                    default_visiter.indices.len(),
                    default_visiter.indices
                );

                default_visiter.to_render_mesh()
            }
            StitchSeamScheme::NeighborConnect => {
                debug!(
                    "chunk_min: {}, voxel num: {}, seam_leaf_nodes size: {} before octree build",
                    seam_chunk_min, seam_voxel_num, octree.get_nodes_num()
                );
                Octree::build_children_nodes_by_clone(&mut octree);
                debug!(
                    "chunk_min: {}, voxel num: {}, seam_leaf_nodes size: {} after octree build",
                    seam_chunk_min, seam_voxel_num, octree.get_nodes_num()
                );
                let seam_data = seam_connect(&mut octree);
                debug!("seam mesh position len: {} indices len: {}", seam_data.positions.len(), seam_data.indices.len());
                seam_data.to_render_mesh()
            }
        };

        if mesh.indices().unwrap().is_empty() {
            return;
        }
        match sender.send(TerrainChunkMeshData {
            entity,
            main_mesh_data: None,
            seam_mesh_data: Some(TerrainChunkSeamMeshData { seam_mesh: mesh }),
        }) {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }
    });
}
