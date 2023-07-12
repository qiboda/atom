use bevy::prelude::*;

use crate::terrain::chunk::TerrainChunk;

use self::{
    bundle::OctreeBundle,
    octree::{make_octree_structure, mark_transitional_faces, Octree, OctreeCellAddress},
};

use super::IsosurfaceExtractionSet;

pub mod address;
pub mod bundle;
pub mod cell;
pub mod def;
pub mod edge;
pub mod face;
pub mod octree;
pub mod point;
pub mod strip;
pub mod tables;

pub struct OctreePlugin;

impl Plugin for OctreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_octree).add_systems(
            Update,
            (make_octree_structure, mark_transitional_faces)
                .chain()
                .in_set(IsosurfaceExtractionSet::BuildOctree),
        );
    }
}

fn add_octree(
    mut commands: Commands,
    chunks: Query<Entity, (Added<TerrainChunk>, Without<Octree>)>,
) {
    for chunk in chunks.iter() {
        commands.entity(chunk).insert(OctreeBundle {
            octree: Octree::default(),
            octree_cells: OctreeCellAddress::default(),
        });
    }
}
