use bevy::prelude::*;

use self::tessellation::tessellation_traversal;

use super::IsosurfaceExtractionSet;

pub mod mesh;
pub mod tessellation;
pub mod vertex_index;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Startup,
            (tessellation_traversal).in_set(IsosurfaceExtractionSet::Meshing),
        );
    }
}
