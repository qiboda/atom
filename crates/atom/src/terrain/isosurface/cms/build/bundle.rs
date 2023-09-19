use bevy::prelude::Bundle;

use super::{cell::Cell, face::Faces};

#[derive(Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub faces: Faces,
}
