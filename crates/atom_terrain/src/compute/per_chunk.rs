//! Per-chunk GPU terrain compute pipeline。
//!
//! WIP: 取代全局 single-grid 方案。每 chunk 33³ voxels（32 active + 1 ghost border），
//! 独立 compute buffer 和 bind group，共享 pipeline 对象。
//!
//! TODO: complete implementation and wire into PerChunkTerrainPlugin

use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;

/// Placeholder resource — actual implementation pending
#[derive(Resource)]
pub struct PerChunkComputePipeline;

/// Placeholder — will initialize per-chunk compute pipeline
pub fn init_per_chunk_compute(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    _render_device: Res<RenderDevice>,
) {
    info!("PerChunkComputePipeline: stub initialized");
}

/// Placeholder — will dispatch compute per chunk
pub fn per_chunk_compute_system() {}

/// Placeholder — will advance chunk states
pub fn advance_chunk_states() {}
