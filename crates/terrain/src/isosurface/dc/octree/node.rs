use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume},
        Vec3A,
    },
    prelude::*,
};
use pqef::Quadric;
use strum::{EnumCount, IntoEnumIterator};

use super::{
    address::{FaceAddress, NodeAddress},
    tables::{FaceIndex, SubNodeIndex, VertexIndex, EDGE_VERTEX_PAIRS},
    OctreeSampler,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum NodeType {
    Branch,
    Leaf,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum VoxelMaterialType {
    Air,
    Block,
}

#[derive(Debug, Component, Reflect, Clone)]
#[reflect(from_reflect = false)]
pub struct Node {
    pub node_type: NodeType,
    pub address: NodeAddress,
    // positions
    pub aabb: Aabb3d,
    pub coord: Vec3A,
    pub vertices_mat_types: [VoxelMaterialType; VertexIndex::COUNT],
    pub conner_sampler_data: [f32; 8],

    pub qef: Option<Quadric>,
    pub qef_error: f32,
    pub vertex_estimate: Vec3,
    pub normal_estimate: Vec3A,
}

impl Node {
    pub fn new(node_type: NodeType, address: NodeAddress) -> Self {
        Self {
            node_type,
            address,
            aabb: Aabb3d::new(Vec3::ZERO, Vec3::ONE),
            coord: Vec3A::ZERO,
            qef: None,
            qef_error: 0.0,
            vertex_estimate: Vec3::ZERO,
            normal_estimate: Vec3A::ZERO,
            vertices_mat_types: [VoxelMaterialType::Air; VertexIndex::COUNT],
            conner_sampler_data: [0.0; 8],
        }
    }

    pub fn get_subnode_aabb(aabb: Aabb3d, subnode_index: SubNodeIndex) -> Aabb3d {
        let center = aabb.center();
        let mut min = Vec3::ZERO;
        let mut max = Vec3::ZERO;

        if subnode_index as u8 & 0b001 == 0b001 {
            min.x = center.x;
            max.x = aabb.max.x;
        } else {
            min.x = aabb.min.x;
            max.x = center.x;
        }

        if subnode_index as u8 & 0b010 == 0b010 {
            min.y = center.y;
            max.y = aabb.max.y;
        } else {
            min.y = aabb.min.y;
            max.y = center.y;
        }

        if subnode_index as u8 & 0b100 == 0b100 {
            min.z = center.z;
            max.z = aabb.max.z;
        } else {
            min.z = aabb.min.z;
            max.z = center.z;
        }

        Aabb3d::new(min, max)
    }

    pub fn get_node_vertex_locations(aabb: Aabb3d) -> [Vec3; VertexIndex::COUNT] {
        let min = aabb.min;
        let max = aabb.max;
        [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
        ]
    }
}

impl Node {
    pub fn get_twin_face_address(&self, face_index: FaceIndex) -> FaceAddress {
        let neighbor_address = self.address.get_neighbor_address(face_index);
        let neighbor_face_index = match face_index {
            FaceIndex::Back => FaceIndex::Front,
            FaceIndex::Front => FaceIndex::Back,
            FaceIndex::Bottom => FaceIndex::Top,
            FaceIndex::Top => FaceIndex::Bottom,
            FaceIndex::Left => FaceIndex::Right,
            FaceIndex::Right => FaceIndex::Left,
        };
        FaceAddress {
            node_address: neighbor_address,
            face_index: neighbor_face_index,
        }
    }
}

impl Node {
    #[inline]
    pub fn estimate_vertex(
        &mut self,
        sdf: &impl OctreeSampler,
        vertices_values: [f32; VertexIndex::COUNT],
        std_dev_pos: f32,
        std_dev_normal: f32,
    ) {
        self.estimate_vertex_mat(vertices_values);
        let qef = Node::estimate_interior_vertex_qef(
            &self.aabb,
            &vertices_values,
            sdf,
            std_dev_pos,
            std_dev_normal,
        );
        self.estimate_vertex_with_qef(qef.0, qef.1, qef.2);
        trace!(
            "estimate_vertex: {:?}, vertex is mass_point: {}",
            self.qef_error,
            self.vertex_estimate == qef.1.into()
        );
    }

    #[inline]
    pub fn estimate_vertex_with_qef(&mut self, qef: Quadric, _mass_point: Vec3A, normal: Vec3A) {
        let p = qef.minimizer();
        let qef_error = qef.residual_l2_error(p);
        self.qef = Some(qef);
        self.qef_error = qef_error;
        self.normal_estimate = normal;
        // if self.qef_error < 0.00001 {
        self.vertex_estimate = p.into();
        // } else {
        //     self.vertex_estimate = mass_point.into();
        // }
    }

    pub fn point_gradient(sdf: &impl OctreeSampler, p: Vec3A, delta: f32) -> Vec3A {
        let h = 0.5 * delta;

        Vec3A::new(
            sdf.sampler_split(p.x + h, p.y, p.z) - sdf.sampler_split(p.x, p.y, p.z),
            sdf.sampler_split(p.x, p.y + h, p.z) - sdf.sampler_split(p.x, p.y, p.z),
            sdf.sampler_split(p.x, p.y, p.z + h) - sdf.sampler_split(p.x, p.y, p.z),
        )
        .normalize()
    }

    pub fn central_gradient(sdf: &impl OctreeSampler, p: Vec3A, delta: f32) -> Vec3A {
        let h = 0.5 * delta;

        Vec3A::new(
            sdf.sampler_split(p.x + h, p.y, p.z) - sdf.sampler_split(p.x - h, p.y, p.z),
            sdf.sampler_split(p.x, p.y + h, p.z) - sdf.sampler_split(p.x, p.y - h, p.z),
            sdf.sampler_split(p.x, p.y, p.z + h) - sdf.sampler_split(p.x, p.y, p.z - h),
        )
        .normalize()
    }

    pub fn estimate_vertex_mat(&mut self, vertices_values: [f32; VertexIndex::COUNT]) {
        // assert!(self.node_type == NodeType::Leaf);
        self.conner_sampler_data = vertices_values;
        for i in VertexIndex::iter() {
            if vertices_values[i as usize] < 0.0 {
                self.vertices_mat_types[i as usize] = VoxelMaterialType::Block;
            } else {
                self.vertices_mat_types[i as usize] = VoxelMaterialType::Air;
            }
        }
    }

    #[allow(dead_code)]
    fn get_positions_and_normals(
        sdf: &impl OctreeSampler,
        center_pos: Vec3A,
        delta: f32,
    ) -> ([Vec3A; 9], [Vec3A; 9]) {
        let mut positions = [Vec3A::ZERO; 9];
        positions[0] = center_pos;
        let mut normals = [Vec3A::ZERO; 9];
        normals[0] = Node::central_gradient(sdf, positions[0], delta);

        let mut index = 1;
        for x in [-1, 1] {
            for y in [-1, 1] {
                for z in [-1, 1] {
                    positions[index] = center_pos
                        + Vec3A::new(delta * x as f32, delta * y as f32, delta * z as f32);
                    normals[index] = Node::central_gradient(sdf, positions[index], delta);
                    index += 1;
                }
            }
        }
        assert_eq!(index, 9);

        (positions, normals)
    }

    pub fn estimate_interior_vertex_qef(
        aabb: &Aabb3d,
        samples: &[f32; VertexIndex::COUNT],
        sdf: &impl OctreeSampler,
        std_dev_pos: f32,
        std_dev_normal: f32,
    ) -> (Quadric, Vec3A, Vec3A) {
        let mut qef = Quadric::default();
        trace!("estimate_interior_vertex_qef, start");
        let corners = Node::get_node_vertex_locations(*aabb);
        let mut avg_normal = Vec3A::ZERO;
        let mut count = 0;
        let mut avg_loc = Vec3A::ZERO;
        for [v1, v2] in EDGE_VERTEX_PAIRS {
            let s1 = samples[v1 as usize];
            let s2: f32 = samples[v2 as usize];

            if (s1 < 0.0 && s2 >= 0.0) || (s1 >= 0.0 && s2 < 0.0) {
                // get the edge vertices.
                let dir = if s2 > s1 { 1.0 } else { -1.0 };
                let mut cross_pos =
                    corners[v1 as usize] + (corners[v2 as usize] - corners[v1 as usize]) * 0.5;
                let mut step = (corners[v2 as usize] - corners[v1 as usize]) / 4.0;
                let mut cross_value = sdf.sampler(cross_pos);
                for _i in 0..8 {
                    if cross_value == 0.0 {
                        break;
                    } else {
                        let offset_dir = if cross_value < 0.0 { dir } else { -dir };
                        cross_pos += offset_dir * step;
                        cross_value = sdf.sampler(cross_pos);
                        step /= 2.0;
                    }
                }

                // let (positions, normals) =
                //     Node::get_positions_and_normals(sdf, cross_pos.into(), delta);
                // let mean_pos = positions.iter().sum::<Vec3A>() / 9.0;
                // let mean_normal = normals.iter().sum::<Vec3A>().normalize();

                // qef += Quadric::probabilistic_plane_quadric_sigma(
                //     mean_pos,
                //     mean_normal,
                //     covariance_matrix(&positions),
                //     covariance_matrix(&normals),
                // );

                let central_normal = Node::central_gradient(sdf, cross_pos.into(), 0.001);
                qef += Quadric::probabilistic_plane_quadric(
                    cross_pos.into(),
                    central_normal,
                    std_dev_pos,
                    std_dev_normal,
                );

                avg_normal += central_normal;
                avg_loc += Vec3A::from(cross_pos);
                count += 1;

                trace!(
                    "estimate_interior_vertex_qef: s1: {}, s2: {}, corners: {} , {}, qef mini: {}, edge_cross_p: {}, \
                     error: {}",
                    s1, s2,
                    corners[v1 as usize],
                    corners[v2 as usize],
                    qef.minimizer(),
                    cross_pos,
                    qef.residual_l2_error(qef.minimizer())
                );
            }
        }

        trace!("estimate_interior_vertex_qef, end");

        avg_normal /= count as f32;
        avg_loc /= count as f32;
        (qef, avg_loc, avg_normal.normalize())
    }
}
