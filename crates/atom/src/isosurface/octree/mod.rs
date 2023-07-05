use bevy::prelude::*;

use self::{
    bundle::OctreeBundle,
    octree::{make_octree_structure, mark_transitional_faces, Octree, OctreeCellAddress},
};

use super::{cms::CMSSet, IsosurfaceExtract};

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
        app.add_systems(
            Startup,
            (start_up, make_octree_structure, mark_transitional_faces)
                .chain()
                .in_set(CMSSet::Octree),
        );
    }
}

fn start_up(mut commands: Commands, parent: Query<Entity, Added<IsosurfaceExtract>>) {
    commands.spawn(OctreeBundle {
        octree: Octree::default(),
        octree_cells: OctreeCellAddress::default(),
    });
}
