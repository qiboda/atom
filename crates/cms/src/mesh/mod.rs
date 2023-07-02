use bevy::prelude::Plugin;

pub mod mesh;

struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_system(Mesh, update_mesh);
    }
}
