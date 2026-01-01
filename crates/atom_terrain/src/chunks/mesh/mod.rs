pub mod channel;
pub mod components;
pub mod compute;
mod handler;
pub mod tables;
pub mod visual;

use bevy::{
    prelude::*,
    render::{RenderApp, extract_component::ExtractComponentPlugin},
};

use crate::{
    chunks::mesh::{
        channel::{TerrainChunkMeshDataReceiver, TerrainChunkMeshDataSender},
        components::TerrainChunkMeshingState,
        compute::mesh_compute::TerrainChunkMeshComputePlugin,
        handler::{receive_chunk_load_requests, receive_chunk_mesh_data},
    },
    terrain::TerrainSystems,
};

/**
 * 注册地形 Chunk 网格化相关的系统
 */
#[derive(Default, Debug)]
pub struct TerrainChunkMeshingPlugin;

impl Plugin for TerrainChunkMeshingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<TerrainChunkMeshingState>::default());

        let (s, r) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkMeshDataReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(TerrainChunkMeshDataSender(s));

        app.add_systems(
            Update,
            receive_chunk_load_requests.in_set(TerrainSystems::GenerateChunk),
        )
        // 因为数据是在渲染的最后发送的，因此在主线程中接收数据的系统放在 First 阶段，衔接比较紧。
        .add_systems(First, receive_chunk_mesh_data);

        // 渲染子系统
        app.add_plugins(TerrainChunkMeshComputePlugin);
    }
}
