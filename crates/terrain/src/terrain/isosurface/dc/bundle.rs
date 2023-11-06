use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Bundle, Component},
    tasks::Task,
};

use crate::terrain::isosurface::mesh::mesh_cache::MeshCache;

use super::{CellOctree, DualContourState};

#[derive(Component, Debug)]
pub struct DualContoring {
    pub octree: Arc<RwLock<CellOctree>>,
    pub mesh_cache: Arc<RwLock<MeshCache>>,
}

#[derive(Component, Debug)]
pub struct DualContoringTask {
    pub state: DualContourState,
    pub task: Option<Task<()>>,
}

#[derive(Bundle, Debug)]
pub struct DualContoringBundle {
    pub dual_contoring: DualContoring,
    pub dual_contoring_task: DualContoringTask,
}
