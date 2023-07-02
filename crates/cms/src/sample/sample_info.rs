use bevy::prelude::Component;
use nalgebra::Vector3;

use super::sample_range_3d::SampleRange3D;

#[derive(Debug, Component, Default)]
pub struct SampleInfo {
    pub samples_size: Vector3<usize>,

    pub sample_data: SampleRange3D<f32>,

    /// one cell location size
    pub offsets: Vector3<f32>,
}
