use bevy::prelude::Plugin;

pub mod mesh;
pub mod vertex_index;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_system(Mesh, update_mesh);
    }
}
