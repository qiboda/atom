pub mod terrain_tracing;

use bevy::{log::BoxedLayer, prelude::*};
use log_layers::LogLayerGuardRes;
use project::project_saved_root_path;
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
