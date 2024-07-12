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
    address::{CellAddress, FaceAddress},
    tables::{FaceIndex, SubCellIndex, VertexIndex, EDGE_VERTEX_PAIRS},
    OctreeSampler,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum CellType {
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
pub struct Cell {
    pub cell_type: CellType,
    pub address: CellAddress,
    pub coord: Vec3A,
    // positions
    #[reflect(ignore)]
    pub aabb: Aabb3d,

    // position sampler values
    pub vertices_samples: [f32; VertexIndex::COUNT],
    pub vertices_mat_types: [VoxelMaterialType; VertexIndex::COUNT],
    pub qef: Option<Quadric>,

    pub qef_error: f32,
    pub vertex_estimate: Vec3,
    pub normal_estimate: Vec3A,
}

impl Cell {
    pub fn new(cell_type: CellType, address: CellAddress) -> Self {
        Self {
            cell_type,
            address,
            aabb: Aabb3d::new(Vec3::ZERO, Vec3::ONE),

            vertices_samples: [0.0; VertexIndex::COUNT],
            qef: None,
            qef_error: 0.0,
            vertex_estimate: Vec3::ZERO,
            coord: Vec3A::ZERO,
            normal_estimate: Vec3A::ZERO,
            vertices_mat_types: [VoxelMaterialType::Air; VertexIndex::COUNT],
        }
    }

    pub fn get_subcell_aabb(aabb: Aabb3d, subcell_index: SubCellIndex) -> Aabb3d {
        let center = aabb.center();
        let mut min = Vec3::ZERO;
        let mut max = Vec3::ZERO;

        if subcell_index as u8 & 0b001 == 0b001 {
            min.x = center.x;
            max.x = aabb.max.x;
        } else {
            min.x = aabb.min.x;
            max.x = center.x;
        }

        if subcell_index as u8 & 0b010 == 0b010 {
            min.y = center.y;
            max.y = aabb.max.y;
        } else {
            min.y = aabb.min.y;
            max.y = center.y;
        }

        if subcell_index as u8 & 0b100 == 0b100 {
            min.z = center.z;
            max.z = aabb.max.z;
        } else {
            min.z = aabb.min.z;
            max.z = center.z;
        }

        Aabb3d::new(min, max)
    }

    pub fn get_cell_vertex_locations(aabb: Aabb3d) -> [Vec3; VertexIndex::COUNT] {
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

impl Cell {
    pub fn get_twin_face_address(&self, face_index: FaceIndex) -> FaceAddress {
        let neighbour_address = self.address.get_neighbour_address(face_index);
        let neighbour_face_index = match face_index {
            FaceIndex::Back => FaceIndex::Front,
            FaceIndex::Front => FaceIndex::Back,
            FaceIndex::Bottom => FaceIndex::Top,
            FaceIndex::Top => FaceIndex::Bottom,
            FaceIndex::Left => FaceIndex::Right,
            FaceIndex::Right => FaceIndex::Left,
        };
        FaceAddress {
            cell_address: neighbour_address,
            face_index: neighbour_face_index,
        }
    }
}

impl Cell {
    #[inline]
    pub fn estimate_vertex(&mut self, sdf: &impl OctreeSampler, precision: f32) {
        self.estimate_vertex_mat();
        let qef =
            Cell::estimate_interior_vertex_qef(&self.aabb, &self.vertices_samples, sdf, precision);
        self.estimate_vertex_with_qef(qef.0, qef.1, qef.2);
        info!(
            "estimate_vertex: {:?}, vertex is mass_point: {}",
            self.qef_error,
            self.vertex_estimate == qef.1.into()
        );
    }

    #[inline]
    pub fn estimate_vertex_with_qef(&mut self, qef: Quadric, mass_point: Vec3A, normal: Vec3A) {
        let p = qef.minimizer();
        let qef_error = qef.residual_l2_error(p);
        self.qef = Some(qef);
        self.qef_error = qef_error;
        self.normal_estimate = normal;
        if self.qef_error < 0.0001 {
            self.vertex_estimate = p.into();
        } else {
            self.vertex_estimate = mass_point.into();
        }
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

    pub fn estimate_vertex_mat(&mut self) {
        assert!(self.cell_type == CellType::Leaf);
        for i in VertexIndex::iter() {
            if self.vertices_samples[i as usize] < 0.0 {
                self.vertices_mat_types[i as usize] = VoxelMaterialType::Block;
            } else {
                self.vertices_mat_types[i as usize] = VoxelMaterialType::Air;
            }
        }
    }

    pub fn estimate_interior_vertex_qef(
        aabb: &Aabb3d,
        samples: &[f32; 8],
        sdf: &impl OctreeSampler,
        precision: f32,
    ) -> (Quadric, Vec3A, Vec3A) {
        let mut qef = Quadric::default();
        info!("estimate_interior_vertex_qef, start");
        let corners = Cell::get_cell_vertex_locations(*aabb);
        let mut avg_normal = Vec3A::ZERO;
        let mut count = 0;
        let mut masspoint = Vec3A::ZERO;
        for [v1, v2] in EDGE_VERTEX_PAIRS {
            let s1 = samples[v1 as usize];
            let s2: f32 = samples[v2 as usize];

            if (s1 < 0.0 && s2 >= 0.0) || (s1 >= 0.0 && s2 < 0.0) {
                // Lerp the edge vertices.
                let dir = if s2 > s1 { 1.0 } else { -1.0 };
                let mut cross_pos =
                    corners[v1 as usize] + (corners[v2 as usize] - corners[v1 as usize]) * 0.5;
                let mut step = (corners[v2 as usize] - corners[v1 as usize]) / 4.0;
                let mut corss_value = sdf.sampler(cross_pos);
                for _i in 0..8 {
                    if corss_value == 0.0 {
                        break;
                    } else {
                        let offset_dir = if corss_value < 0.0 { dir } else { -dir };
                        cross_pos += offset_dir * step;
                        corss_value = sdf.sampler(cross_pos);
                        step /= 2.0;
                    }
                }

                let delta = aabb.half_size().x;
                let mut normal = Cell::central_gradient(sdf, cross_pos.into(), delta);
                if normal.is_nan() {
                    error!("estimate_interior_vertex_qef normal is nan");
                    normal = (aabb.center() - Vec3A::from(cross_pos)).normalize();
                }

                avg_normal += normal;
                masspoint += Vec3A::from(cross_pos);
                count += 1;

                qef += Quadric::probabilistic_plane_quadric(
                    cross_pos.into(),
                    normal,
                    precision * delta,
                    precision,
                );

                info!(
                    "estimate_interior_vertex_qef: s1: {}, s2: {}, corners: {} , {}, edge_cross_p: {}, normal: {}, delta: {} \
                    qef mini: {}, error: {}",
                    s1, s2,
                    corners[v1 as usize],
                    corners[v2 as usize],
                    cross_pos,
                    normal,
                    delta,
                    qef.minimizer(),
                    qef.residual_l2_error(qef.minimizer())
                );
            }
        }

        info!("estimate_interior_vertex_qef, end");

        avg_normal /= count as f32;
        masspoint /= count as f32;
        (qef, masspoint, avg_normal.normalize())
    }
}
