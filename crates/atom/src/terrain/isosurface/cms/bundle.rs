use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Bundle, Component},
    tasks::Task,
};

use crate::terrain::isosurface::{
    meshing::mesh::MeshCache, octree::octree::Octree, sample::surface_sampler::SurfaceSampler,
    IsosurfaceExtractionState,
};

#[derive(Debug, Component)]
pub struct CMSComponent {
    pub mesh_cache: Arc<RwLock<MeshCache>>,
    pub octree: Arc<RwLock<Octree>>,
    pub surface_sampler: Arc<RwLock<SurfaceSampler>>,
}

#[derive(Debug, Component, Default)]
pub struct CMSTask {
    pub state: IsosurfaceExtractionState,
    pub task: Option<Task<()>>,
}

#[derive(Debug, Bundle)]
pub struct CMSBundle {
    pub cms: CMSComponent,
    pub task: CMSTask,
}
