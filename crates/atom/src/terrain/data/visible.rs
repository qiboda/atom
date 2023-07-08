use bevy::prelude::{Component, Vec3};

#[derive(Debug, Hash, PartialEq, Default, Eq, Clone, Copy)]
pub enum VisibleAxis {
    #[default]
    Unqiue,
    Positive,
    Negative,
    Full,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct VisibleDirection {
    pub x: VisibleAxis,
    pub y: VisibleAxis,
    pub z: VisibleAxis,
}

impl VisibleDirection {
    pub fn new(x: VisibleAxis, y: VisibleAxis, z: VisibleAxis) -> Self {
        Self { x, y, z }
    }

    pub fn is_full_visible(&self) -> bool {
        self.x == VisibleAxis::Full && self.y == VisibleAxis::Full && self.z == VisibleAxis::Full
    }
}

#[derive(Debug, Hash, Default, PartialEq, Eq, Clone, Component)]
pub enum TerrainVisibility {
    Visible(VisibleDirection),
    #[default]
    Hidden,
}

#[derive(Debug, Component)]
pub struct VisibleTerrainRange {
    pub min: Vec3,
    pub max: Vec3,
}
