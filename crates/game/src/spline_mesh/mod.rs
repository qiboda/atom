use bevy::{math::VectorSpace, prelude::*, render::mesh::VertexAttributeValues};

#[derive(Component, Debug)]
pub struct SplineMesh {
    ref_mesh: Mesh,
    control_points: Vec<Vec3>,
    catmull_rom_spline: CubicCardinalSpline<Vec3>,

    position: Vec<Vec3>,
    normal: Vec<Vec3>,
    uv: Vec<Vec2>,

    indices: Vec<u32>,

    shape_2d_pos: Vec<Vec3>,
    shape_2d_uv: Vec<Vec2>,
}

impl SplineMesh {
    pub fn add_point(&mut self, point: Vec3) {
        self.control_points.push(point);
        let spline = CubicCardinalSpline::new_catmull_rom(self.control_points.clone());
        let curve = spline.to_curve();

        let positions = curve.iter_positions(1);
    }

    pub fn build_spline_mesh(&mut self) {}
}
