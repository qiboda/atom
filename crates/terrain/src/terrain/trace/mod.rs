pub mod terrain_tracing;

use bevy::prelude::*;
use serde_json::json;
use terrain_player_client::{EdgeData, OrderType, TriangleData, VertexData};

pub struct TerrainTracePlugin;

// 改为写在log初始化之前。
impl Plugin for TerrainTracePlugin {
    fn build(&self, _app: &mut App) {

        // let filter = EnvFilter::new(TERRAIN_TRACE_TARGET.to_owned() + "=trace");
        //
        // let trace_path = project_saved_root_path().join("trace");
        // let appender = tracing_appender::rolling::never(trace_path, "trace");
        // let (non_blocking, _guard) = tracing_appender::non_blocking(appender);
        //
        // let fmt = tracing_subscriber::fmt()
        //     .with_writer(non_blocking)
        //     .with_env_filter(filter);
        //
        // let _ = tracing_subscriber::registry().with_subscriber(fmt);
    }
}

pub const TERRAIN_TRACE_TARGET: &str = "terrain_trace";

#[macro_export]
macro_rules! terrain_trace_span {
    ($name:expr) => {
        bevy::utils::tracing::trace_span!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, $name)
    };
    ($name:expr, $($fields:tt)*) => {
        bevy::utils::tracing::trace_span!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, $name, $($fields)*)
    };
    (parent: $parent:expr, $name:expr) => {
        bevy::utils::tracing::trace_span!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, parent: $parent, $name)
    };
    (parent: $parent:expr, $name:expr, $($fields:tt)*) => {
        bevy::utils::tracing::trace_span!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, parent: $parent, $name, $($fields)*)
    };
}

#[macro_export]
macro_rules! terrain_trace {
    // Name / target.
    ({ $($field:tt)* }, $($arg:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, { $($field)* }, $($arg)*)
    );
    ($($k:ident).+ $($field:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, $($k).+ $($field)*)
    );
    (?$($k:ident).+ $($field:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, ?$($k).+ $($field)*)
    );
    (%$($k:ident).+ $($field:tt)* ) => (
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, %$($k).+ $($field)*)
    );
    ($($arg:tt)+ ) => (
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, $($arg)+)
    );
}

pub fn terrain_trace_vertex(index: usize, location: Vec3) {
    let order_type = OrderType::Vertex(VertexData { index, location });
    let order_type = serde_json::to_string(&json!(order_type)).unwrap();
    terrain_trace!(order_type);
}

pub fn terrain_trace_edge(start_location: Vec3, end_location: Vec3) {
    let order_type = OrderType::Edge(EdgeData {
        start_location,
        end_location,
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
