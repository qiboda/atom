use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Bundle, Component},
    tasks::Task,
};

use crate::terrain::isosurface::mesh::mesh_cache::MeshCache;

use super::{CellOctree, DualContourState};

#[derive(Component, Debug)]
pub struct DualContouring {
    pub octree: Arc<RwLock<CellOctree>>,
    pub mesh_cache: Arc<RwLock<MeshCache>>,
}

#[derive(Component, Debug)]
pub struct DualContouringTask {
    pub state: DualContourState,
    pub task: Option<Task<()>>,
}

#[derive(Bundle, Debug)]
pub struct DualContouringBundle {
    pub dual_contouring: DualContouring,
    pub dual_contouring_task: DualContouringTask,
}
