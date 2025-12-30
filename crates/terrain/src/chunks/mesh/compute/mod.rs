pub mod bind_group;
pub mod buffer;
pub mod data;
pub mod mesh_compute;
pub mod node;
pub mod pipelines;

use bevy::{app::App, render::RenderApp};

pub fn terrain_chunk_meshing_compute_systems(app: &mut App) {
    let Some(_render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    // mesh_compute::compute_pipeline_systems(render_app);
    // mesh_compute::compute_node_systems(render_app);
}
