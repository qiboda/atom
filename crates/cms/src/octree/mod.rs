use bevy::prelude::*;

use self::{
    bundle::OctreeBundle,
    octree::{make_octree_structure, mark_transitional_faces, Octree, OctreeCellAddress},
};

pub mod address;
pub mod bundle;
pub mod cell;
pub mod def;
pub mod edge;
pub mod edge_block;
pub mod face;
pub mod octree;
pub mod point;
pub mod strip;
pub mod tables;
pub mod vertex;

pub struct OctreePlugin;

impl Plugin for OctreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_up);
        app.add_systems(Update, make_octree_structure);
        app.add_systems(Update, mark_transitional_faces);
    }
}

fn start_up(mut commands: Commands) {
    commands.spawn(OctreeBundle {
        octree: Octree::default(),
        octree_cells: OctreeCellAddress::default(),
    });
}
