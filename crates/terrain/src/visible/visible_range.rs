use bevy::prelude::{Component, Vec3};

#[derive(Debug, Component)]
pub struct VisibleTerrainRange {
    min: Vec3,
    max: Vec3,
}

impl VisibleTerrainRange {
    pub fn new(range: Vec3) -> Self {
        let half = range * 0.5;
        Self {
            min: -half,
            max: half,
        }
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn is_in_range(&self, x: f32, y: f32, z: f32) -> bool {
        self.min.x <= x
            && x <= self.max.x
            && self.min.y <= y
            && y <= self.max.y
            && self.min.z <= z
            && z <= self.max.z
    }
}
