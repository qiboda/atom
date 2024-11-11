use bevy::{
    math::Vec3,
    prelude::Mesh,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
    utils::HashMap,
};
use wgpu::PrimitiveTopology;

use crate::{
    isosurface::{select_voxel_biome, IsosurfaceSide},
    lod::{morton_code::MortonCode, morton_code_neighbor::MortonCodeNeighbor},
    materials::terrain_material::BIOME_VERTEX_ATTRIBUTE,
    tables::{EdgeIndex, FaceIndex},
};

use super::octree::{node::Node, Octree, OctreeLevel};

#[derive(Default)]
pub struct SeamConnectData {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub biomes: Vec<u32>,
    pub indices: Vec<u32>,
    pub address_vertex_id_map: HashMap<MortonCode, u32>,
}

impl SeamConnectData {
    pub fn to_render_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(BIOME_VERTEX_ATTRIBUTE, self.biomes);
        mesh.insert_indices(Indices::U32(self.indices));

        // mesh.compute_normals();
        // match mesh.generate_tangents() {
        //     Ok(_) => {}
        //     Err(e) => {
        //         warn!("generate_tangents error: {:?}", e);
        //     }
        // }

        mesh
    }
}

pub fn connect_x(
    code_0: MortonCode,
    node_0: &Node,
    level: &OctreeLevel,
    data: &mut SeamConnectData,
) {
    // 打开这个判断，会导致洞
    // if node_0.vertices_side_types[6] == node_0.vertices_side_types[7] {
    //     return;
    // }

    let mut nodes = vec![node_0];

    let front_code = code_0.get_neighbor_face_morton_code(FaceIndex::Front);
    let front_top_code = code_0.get_neighbor_edge_morton_code(EdgeIndex::XAxisY1Z1);
    let top_code = code_0.get_neighbor_face_morton_code(FaceIndex::Top);

    if let Some(x) = front_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if let Some(x) = front_top_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if let Some(x) = top_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if nodes.len() < 3 {
        return;
    }

    if nodes.len() == 4 {
        let index_0 = *data.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let index_1 = *data.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let index_2 = *data.address_vertex_id_map.get(&nodes[2].address).unwrap();
        let index_3 = *data.address_vertex_id_map.get(&nodes[3].address).unwrap();

        if node_0.vertices_side_types[7] == IsosurfaceSide::Inside {
            data.indices.push(index_0);
            data.indices.push(index_1);
            data.indices.push(index_2);

            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_3);
        } else {
            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_1);

            data.indices.push(index_0);
            data.indices.push(index_3);
            data.indices.push(index_2);
        }
    }
    // == 3
    else {
        let index_0 = *data.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let index_1 = *data.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let index_2 = *data.address_vertex_id_map.get(&nodes[2].address).unwrap();

        if node_0.vertices_side_types[7] == IsosurfaceSide::Inside {
            data.indices.push(index_0);
            data.indices.push(index_1);
            data.indices.push(index_2);
        } else {
            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_1);
        }
    }
}

pub fn connect_y(
    code_0: MortonCode,
    node_0: &Node,
    level: &OctreeLevel,
    data: &mut SeamConnectData,
) {
    // 打开这个判断，会导致洞
    // if node_0.vertices_side_types[5] == node_0.vertices_side_types[7] {
    //     return;
    // }

    let mut nodes = vec![node_0];

    let right_code = code_0.get_neighbor_face_morton_code(FaceIndex::Right);
    let right_front_code = code_0.get_neighbor_edge_morton_code(EdgeIndex::YAxisX1Z1);
    let front_code = code_0.get_neighbor_face_morton_code(FaceIndex::Front);

    if let Some(x) = right_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if let Some(x) = right_front_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if let Some(x) = front_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if nodes.len() < 3 {
        return;
    }

    if nodes.len() == 4 {
        let index_0 = *data.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let index_1 = *data.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let index_2 = *data.address_vertex_id_map.get(&nodes[2].address).unwrap();
        let index_3 = *data.address_vertex_id_map.get(&nodes[3].address).unwrap();

        if node_0.vertices_side_types[7] == IsosurfaceSide::Inside {
            data.indices.push(index_0);
            data.indices.push(index_1);
            data.indices.push(index_2);

            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_3);
        } else {
            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_1);

            data.indices.push(index_0);
            data.indices.push(index_3);
            data.indices.push(index_2);
        }
    }
    // == 3
    else {
        let index_0 = *data.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let index_1 = *data.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let index_2 = *data.address_vertex_id_map.get(&nodes[2].address).unwrap();

        if node_0.vertices_side_types[7] == IsosurfaceSide::Inside {
            data.indices.push(index_0);
            data.indices.push(index_1);
            data.indices.push(index_2);
        } else {
            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_1);
        }
    }
}

pub fn connect_z(
    code_0: MortonCode,
    node_0: &Node,
    level: &OctreeLevel,
    data: &mut SeamConnectData,
) {
    // 打开这个判断，会导致洞
    // if node_0.vertices_side_types[3] == node_0.vertices_side_types[7] {
    //     return;
    // }

    let mut nodes = vec![node_0];

    let right_code = code_0.get_neighbor_face_morton_code(FaceIndex::Right);
    let right_top_code = code_0.get_neighbor_edge_morton_code(EdgeIndex::ZAxisX1Y1);
    let top_code = code_0.get_neighbor_face_morton_code(FaceIndex::Top);

    if let Some(x) = right_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if let Some(x) = right_top_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if let Some(x) = top_code
        .map(|x| level.address_node_map.get(&x))
        .unwrap_or(None)
    {
        nodes.push(x)
    }

    if nodes.len() < 3 {
        return;
    }

    if nodes.len() == 4 {
        let index_0 = *data.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let index_1 = *data.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let index_2 = *data.address_vertex_id_map.get(&nodes[2].address).unwrap();
        let index_3 = *data.address_vertex_id_map.get(&nodes[3].address).unwrap();

        if node_0.vertices_side_types[3] == IsosurfaceSide::Inside {
            data.indices.push(index_0);
            data.indices.push(index_1);
            data.indices.push(index_2);

            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_3);
        } else {
            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_1);

            data.indices.push(index_0);
            data.indices.push(index_3);
            data.indices.push(index_2);
        }
    }
    // == 3
    else {
        let index_0 = *data.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let index_1 = *data.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let index_2 = *data.address_vertex_id_map.get(&nodes[2].address).unwrap();

        if node_0.vertices_side_types[3] == IsosurfaceSide::Inside {
            data.indices.push(index_0);
            data.indices.push(index_1);
            data.indices.push(index_2);
        } else {
            data.indices.push(index_0);
            data.indices.push(index_2);
            data.indices.push(index_1);
        }
    }
}

pub fn seam_connect(octree: &mut Octree) -> SeamConnectData {
    let mut data = SeamConnectData::default();

    let level = octree.levels.last().unwrap();

    for (_, node) in level.address_node_map.iter() {
        let position = node.vertex_estimate;
        let index = data.positions.iter().position(|x| *x == position);
        match index {
            Some(index) => {
                data.address_vertex_id_map
                    .insert(node.address, index as u32);
            }
            None => {
                let index = data.positions.len();
                data.positions.push(position);
                data.normals.push(node.normal_estimate);
                data.biomes
                    .push(select_voxel_biome(node.vertices_biomes) as u32);
                data.address_vertex_id_map
                    .insert(node.address, index as u32);
            }
        }
    }

    for (code_0, node_0) in level.address_node_map.iter() {
        // x axis, 右边，上边，和右上角的node。
        connect_x(*code_0, node_0, level, &mut data);
        // y axis
        connect_y(*code_0, node_0, level, &mut data);
        // z axis
        connect_z(*code_0, node_0, level, &mut data);
    }

    data
}
