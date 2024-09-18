use bevy::{prelude::*, render::primitives::Aabb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplineMeshFillType {
    Once,
    Repeat,
    StretchToInterval,
}

#[derive(Debug, Clone)]
pub struct SplinePoints {
    pub translation: Vec<Vec3>,
    pub translate_cubic_curve: CubicCurve<Vec3>,

    // TODO: Rotation to use Quat
    pub rotation: Vec<Vec3>,
    pub rotation_cubic_curve: CubicCurve<Vec3>,

    pub scale: Vec<Vec3>,
    pub scale_cubic_curve: CubicCurve<Vec3>,

    pub segment_sample_step: usize,
    pub segment_length: Vec<f32>,
}

impl SplinePoints {
    pub fn push_point(&mut self, point: Transform) {
        self.translation.push(point.translation);
        let rotation = point.rotation.to_euler(EulerRot::XYZ);
        self.rotation
            .push(Vec3::new(rotation.0, rotation.1, rotation.2));
        self.scale.push(point.scale);
    }

    pub fn remove_point(&mut self, index: usize) {
        self.translation.remove(index);
        self.rotation.remove(index);
        self.scale.remove(index);
    }

    pub fn get_curve_segment_len(&self, segment: usize) -> f32 {
        *self.segment_length.get(segment).unwrap_or(&0.0)
    }

    pub fn get_total_length(&self) -> f32 {
        self.segment_length.iter().sum()
    }

    pub fn get_segment_t(&self, length: f32) -> Option<f32> {
        let mut total_length = 0.0;
        for (index, segment_length) in self.segment_length.iter().enumerate() {
            total_length += segment_length;

            if total_length >= length {
                return Some(
                    index as f32 + (length - (total_length - segment_length)) / segment_length,
                );
            }
        }

        None
    }

    pub fn bake_sample(&mut self, segment_step: usize) {
        self.segment_sample_step = segment_step;
        self.segment_length.clear();

        let interval = 1.0 / segment_step as f32;

        for segment in self.translate_cubic_curve.segments.iter() {
            let mut last_position = Vec3::ZERO;
            let mut length = 0.0;
            for i in 0..segment_step {
                let t = i as f32 * interval;
                let position = segment.position(t);
                length += (position - last_position).length();
                last_position = position;
            }
            self.segment_length.push(length);
        }
    }

    pub fn generate_curve(&mut self) {
        let catmull_rom_spline = CubicCardinalSpline::new_catmull_rom(self.translation.clone());
        self.translate_cubic_curve = catmull_rom_spline.to_curve();

        let catmull_rom_spline = CubicCardinalSpline::new_catmull_rom(self.rotation.clone());
        self.rotation_cubic_curve = catmull_rom_spline.to_curve();

        let catmull_rom_spline = CubicCardinalSpline::new_catmull_rom(self.scale.clone());
        self.scale_cubic_curve = catmull_rom_spline.to_curve();
    }
}

#[derive(Debug, Clone)]
pub enum SplineMeshAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone)]
pub struct MeshRef {
    ref_mesh: Mesh,

    // min and max
    aabb: Aabb,
}

impl MeshRef {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            ref_mesh: mesh,
            aabb: Aabb::from_min_max(Vec3::ZERO, Vec3::ZERO),
        }
    }

    pub fn compute_aabb(&mut self) {
        self.aabb = self.ref_mesh.compute_aabb().unwrap();
    }
}

#[derive(Component, Debug)]
pub struct SplineMesh {
    ref_mesh: MeshRef,

    spline_points: SplinePoints,

    fill_type: SplineMeshFillType,
    forward_axis: SplineMeshAxis,
}

impl SplineMesh {
    pub fn set_fill_type(&mut self, fill_type: SplineMeshFillType) {
        self.fill_type = fill_type;
    }

    pub fn set_forward_axis(&mut self, axis: SplineMeshAxis) {
        self.forward_axis = axis;
    }

    pub fn set_ref_mesh(&mut self, mesh: Mesh) {
        self.ref_mesh = MeshRef::new(mesh);
    }

    pub fn add_point(&mut self, point: Transform) {
        self.spline_points.push_point(point);
    }

    pub fn remove_points(&mut self, index: usize) {
        self.spline_points.remove_point(index);
    }

    pub fn generate_curve(&mut self) {
        self.spline_points.generate_curve();
    }

    pub fn build_spline_mesh(&mut self) {
        match self.fill_type {
            SplineMeshFillType::Once => {
                for (index, _segment) in self
                    .spline_points
                    .translate_cubic_curve
                    .segments
                    .iter()
                    .enumerate()
                {
                    // todo: 需要修复。
                    self.bent_mesh(index as f32);
                }
            }
            SplineMeshFillType::Repeat => {
                let total_length = self.spline_points.get_total_length();
                let mesh_length = match self.forward_axis {
                    SplineMeshAxis::X => self.ref_mesh.aabb.max().x - self.ref_mesh.aabb.min().x,
                    SplineMeshAxis::Y => self.ref_mesh.aabb.max().y - self.ref_mesh.aabb.min().y,
                    SplineMeshAxis::Z => self.ref_mesh.aabb.max().z - self.ref_mesh.aabb.min().z,
                };

                let mesh_num = (total_length / mesh_length).ceil() as usize;
                for i in 0..mesh_num {
                    let t = self.spline_points.get_segment_t(i as f32 * mesh_length);
                    if let Some(t) = t {
                        self.bent_mesh(t);
                    }
                }
            }
            SplineMeshFillType::StretchToInterval => {
                for (index, _segment) in self
                    .spline_points
                    .translate_cubic_curve
                    .segments
                    .iter()
                    .enumerate()
                {
                    self.bent_mesh(index as f32);
                }
            }
        }
    }

    fn bent_mesh(&self, t_offset: f32) -> Mesh {
        let locations = self
            .ref_mesh
            .ref_mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .iter()
            .map(|v| Vec3::from_slice(v))
            .collect::<Vec<Vec3>>();

        let aabb = self.ref_mesh.aabb;

        let mut new_locations = vec![];
        for location in locations.iter() {
            let new_location = match self.forward_axis {
                SplineMeshAxis::X => {
                    let width = aabb.max().x - aabb.min().x;
                    // TODO: 需要乘以自身占据的比例
                    let location_t = (location.x - aabb.min().x) / width;
                    let curve_location = self
                        .spline_points
                        .translate_cubic_curve
                        .position(t_offset + location_t);

                    curve_location + Vec3::new(0.0, location.y, location.z)
                }
                SplineMeshAxis::Y => {
                    let width = aabb.max().y - aabb.min().y;
                    let location_t = (location.y - aabb.min().y) / width;
                    let curve_location = self
                        .spline_points
                        .translate_cubic_curve
                        .position(t_offset + location_t);

                    curve_location + Vec3::new(location.x, 0.0, location.z)
                }
                SplineMeshAxis::Z => {
                    let width = aabb.max().z - aabb.min().z;
                    let location_t = (location.z - aabb.min().z) / width;
                    let curve_location = self
                        .spline_points
                        .translate_cubic_curve
                        .position(t_offset + location_t);

                    curve_location + Vec3::new(location.x, location.y, 0.0)
                }
            };
            new_locations.push(new_location);
        }

        let mut new_mesh = self.ref_mesh.ref_mesh.clone();
        new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_locations);

        new_mesh
    }
}
