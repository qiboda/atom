use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume},
        Vec3A,
    },
    prelude::*,
};
use pqef::Quadric;
use strum::EnumCount;

use super::{
    address::{CellAddress, FaceAddress},
    tables::{FaceIndex, SubCellIndex, VertexIndex, EDGE_VERTEX_PAIRS},
    OctreeSampler,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum CellType {
    Branch,
    PseudoLeaf,
    RealLeaf,
}

#[derive(Debug, Component, Reflect, Clone)]
#[reflect(from_reflect = false)]
pub struct Cell {
    pub cell_type: CellType,
    pub address: CellAddress,
    #[reflect(ignore)]
    pub aabb: Aabb3d,

    pub vertices_samples: [f32; VertexIndex::COUNT],
    pub regularized_qef: Option<Quadric>,
    pub exact_qef: Option<Quadric>,

    pub qef_error: f32,
    pub vertex_estimate: Vec3,
}

impl Cell {
    pub fn new(cell_type: CellType, address: CellAddress, aabb: Aabb3d) -> Self {
        Self {
            cell_type,
            address,
            aabb,

            vertices_samples: [0.0; VertexIndex::COUNT],
            regularized_qef: None,
            exact_qef: None,
            qef_error: 0.0,
            vertex_estimate: Vec3::ZERO,
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
        let (regularized_qef, exact_qef) =
            Cell::estimate_interior_vertex_qef(&self.aabb, &self.vertices_samples, sdf, precision);
        self.estimate_vertex_with_qef(&regularized_qef, &exact_qef);
    }

    #[inline]
    pub fn estimate_vertex_with_qef(&mut self, regularized_qef: &Quadric, exact_qef: &Quadric) {
        let p = regularized_qef.minimizer();
        self.exact_qef = Some(*exact_qef);
        self.regularized_qef = Some(*regularized_qef);
        self.qef_error = exact_qef.residual_l2_error(p);
        self.vertex_estimate = p.into();
    }

    pub fn central_gradient(sdf: &impl OctreeSampler, p: Vec3A, delta: f32) -> Vec3A {
        let h = 0.5 * delta;
        Vec3A::new(
            sdf.sampler_split(p.x + h, p.y, p.z) - sdf.sampler_split(p.x - h, p.y, p.z),
            sdf.sampler_split(p.x, p.y + h, p.z) - sdf.sampler_split(p.x, p.y - h, p.z),
            sdf.sampler_split(p.x, p.y, p.z + h) - sdf.sampler_split(p.x, p.y, p.z - h),
        ) / delta
    }

    pub fn estimate_interior_vertex_qef(
        aabb: &Aabb3d,
        samples: &[f32; 8],
        sdf: &impl OctreeSampler,
        precision: f32,
    ) -> (Quadric, Quadric) {
        let mut regularized_qef = Quadric::default();
        let mut exact_qef = Quadric::default();

        let corners = Cell::get_cell_vertex_locations(*aabb);
        for [v1, v2] in EDGE_VERTEX_PAIRS {
            let s1 = samples[v1 as usize];
            let s2 = samples[v2 as usize];
            if (s1 < 0.0) != (s2 < 0.0) && (s1 > 0.0) != (s2 > 0.0) {
                // Lerp the edge vertices.
                let diff = s2 - s1;
                let s1_lerp = s2 / diff;
                let s2_lerp = -s1 / diff;
                let edge_cross_p: Vec3A =
                    (s1_lerp * corners[v1 as usize] + s2_lerp * corners[v2 as usize]).into();

                let delta = aabb.half_size().x * 2.0;
                let normal = Cell::central_gradient(sdf, edge_cross_p, delta).normalize();

                regularized_qef += Quadric::probabilistic_plane_quadric(
                    edge_cross_p,
                    normal,
                    precision * delta,
                    precision,
                );

                exact_qef += Quadric::plane_quadric(edge_cross_p, normal);
            }
        }

        (regularized_qef, exact_qef)
    }
}
