//! GPU indirect terrain render pipeline.
//!
//! Replaces CPU readback → Mesh3d with direct GPU indirect draw from storage buffers.
//! The `GlobalMeshPool` vertex/index/indirect buffers are used directly as vertex/index/indirect
//! GPU resources, bypassing CPU mesh construction for rendering.

mod indirect;

pub use indirect::IndirectTerrainRenderPlugin;
