use bevy::{prelude::*};

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
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, %$($k).+ $($field)+)
    );
    ($($arg:tt)+ ) => (
        bevy::utils::tracing::trace!(target: $crate::terrain::trace::TERRAIN_TRACE_TARGET, $($arg)+)
    );
}

pub fn terrain_trace_vertex(index: usize, location: Vec3) {
    terrain_trace!(index, ?location, "vertex");
}

pub fn terrain_trace_line(start: Vec3, end: Vec3) {
    terrain_trace!(?start, ?end, "line");
}

pub fn terrain_trace_triangle(t1: usize, t2: usize, t3: usize) {
    terrain_trace!(t1, t2, t3, "triangle");
}
