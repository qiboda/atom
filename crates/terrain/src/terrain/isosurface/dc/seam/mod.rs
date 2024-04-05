pub mod seam_octree;

use bevy::{prelude::*, utils::HashMap};
use terrain_core::chunk::coords::TerrainChunkCoord;


#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub(crate) struct SeamConnect {
    pub a: TerrainChunkCoord,
    pub b: TerrainChunkCoord,
}

#[derive(Debug, Default, Resource)]
pub(crate) struct AllSeams {
    pub(crate) seams: HashMap<SeamConnect, seam_octree::SeamOctree>,
}
