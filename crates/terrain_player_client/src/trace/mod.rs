pub mod terrain_tracing;

use crate::order::{LineData, OrderType, TriangleData, VertexData};
use bevy::{log::BoxedLayer, prelude::*, utils::tracing};
use log_layers::LogLayerGuardRes;
use project::project_saved_root_path;
use serde_json::json;
use terrain_core::chunk::coords::TerrainChunkCoord;
use terrain_tracing::TerrainLayer;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer};

const TERRAIN_TRACE_TARGET: &str = "terrain_trace";

pub fn terrain_layer(app: &mut App) -> Option<BoxedLayer> {
    let saved_path = project_saved_root_path();

    let terrain_filter = EnvFilter::new(TERRAIN_TRACE_TARGET.to_owned() + "=trace");

    let trace_path = saved_path.join("trace");
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("terrain_trace")
        .build(trace_path)
        .unwrap();
    let (non_blocking, terrain_worker_guard) = tracing_appender::non_blocking(appender);
    let terrain_layer = TerrainLayer::new()
        .with_pretty(false)
        .with_writer(non_blocking)
        .with_filter(terrain_filter);

    let mut log_layer_res = app
        .world_mut()
        .get_resource_mut::<LogLayerGuardRes>()
        .expect("log layer res is None");
    log_layer_res.worker_guard_vec.push(terrain_worker_guard);

    Some(terrain_layer.boxed())
}

#[macro_export]
macro_rules! terrain_trace_span {
    ($name:expr) => {
        bevy::utils::tracing::trace_span!(target: $crate::trace::TERRAIN_TRACE_TARGET, $name)
    };
    ($name:expr, $($fields:tt)*) => {
        bevy::utils::tracing::trace_span!(target: $crate::trace::TERRAIN_TRACE_TARGET, $name, $($fields)*)
    };
    (parent: $parent:expr, $name:expr) => {
        bevy::utils::tracing::trace_span!(target: $crate::trace::TERRAIN_TRACE_TARGET, parent: $parent, $name)
    };
    (parent: $parent:expr, $name:expr, $($fields:tt)*) => {
        bevy::utils::tracing::trace_span!(target: $crate::trace::TERRAIN_TRACE_TARGET, parent: $parent, $name, $($fields)*)
    };
}

#[macro_export]
macro_rules! terrain_trace {
    // Name / target.
    ({ $($field:tt)* }, $($arg:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::trace::TERRAIN_TRACE_TARGET, { $($field)* }, $($arg)*)
    );
    ($($k:ident).+ $($field:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::trace::TERRAIN_TRACE_TARGET, $($k).+ $($field)*)
    );
    (?$($k:ident).+ $($field:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::trace::TERRAIN_TRACE_TARGET, ?$($k).+ $($field)*)
    );
    (%$($k:ident).+ $($field:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::trace::TERRAIN_TRACE_TARGET, %$($k).+ $($field)*)
    );
    ($($arg:tt)+ ) => (
        bevy::utils::tracing::trace!(target: $crate::trace::TERRAIN_TRACE_TARGET, $($arg)+)
    );
}

pub fn terrain_chunk_trace_span(terrain_chunk_coord: &TerrainChunkCoord) -> tracing::Span {
    let terrain_chunk_coord = serde_json::to_string(&json!(terrain_chunk_coord)).unwrap();
    terrain_trace_span!("terrain_chunk_trace", terrain_chunk_coord)
}

pub fn terrain_trace_vertex(index: usize, location: Vec3) {
    let order_type = OrderType::Vertex(VertexData { index, location });
    let order_type = serde_json::to_string(&json!(order_type)).unwrap();
    terrain_trace!(order_type);
}

pub fn terrain_trace_edge(start_index: usize, end_index: usize) {
    let order_type = OrderType::Line(LineData {
        start_index,
        end_index,
    });

    let order_type = serde_json::to_string(&json!(order_type)).unwrap();
    terrain_trace!(order_type);
}

pub fn terrain_trace_triangle(t1: usize, t2: usize, t3: usize) {
    let order_type = OrderType::Triangle(TriangleData {
        vertex_index_0: t1,
        vertex_index_1: t2,
        vertex_index_2: t3,
    });
    let order_type = serde_json::to_string(&json!(order_type)).unwrap();
    terrain_trace!(order_type);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_trace() {
        let order_type = OrderType::Vertex(VertexData {
            index: 1,
            location: Vec3::new(1.0, 2.0, 3.0),
        });
        let json = serde_json::to_string(&order_type).unwrap();
        println!("{}", json);
    }
}
