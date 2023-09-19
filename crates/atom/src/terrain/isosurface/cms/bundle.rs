use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Bundle, Component, UVec3},
    tasks::Task,
    utils::HashMap,
};

use crate::terrain::isosurface::{mesh::mesh_cache::MeshCache, IsosurfaceExtractionState};

use super::{
    build::octree::Octree, meshing::vertex_index::VertexIndices,
    sample::surface_sampler::SurfaceSampler,
};

#[derive(Debug, Default)]
pub struct CMSVertexIndexInfo {
    pub vertex_index_info: HashMap<UVec3, VertexIndices>,
}

#[derive(Debug, Component)]
pub struct CMSComponent {
    pub vertex_index_info: Arc<RwLock<CMSVertexIndexInfo>>,
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
