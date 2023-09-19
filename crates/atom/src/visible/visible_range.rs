use bevy::prelude::{Component, Vec3};

#[derive(Debug, Component)]
pub struct VisibleTerrainRange {
    pub min: Vec3,
    pub max: Vec3,
}
