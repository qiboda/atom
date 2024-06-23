use bevy::{log::BoxedLayer, prelude::*};
use project::project_saved_root_path;
use tracing_subscriber::{EnvFilter, Layer};

use crate::LogLayerRes;

pub fn file_layer(app: &mut App) -> Option<BoxedLayer> {
    let saved_path = project_saved_root_path();

    let file_filter = EnvFilter::new("info");

    let file_appender = tracing_appender::rolling::daily(saved_path.join("logs"), "log"); // This should be user configurable
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
    let file_fmt_layer = tracing_subscriber::fmt::Layer::default()
        .with_ansi(false) // disable terminal color escape sequences
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .with_writer(non_blocking)
        .with_filter(file_filter);

    let mut log_layer_res = app
        .world_mut()
        .get_resource_mut::<LogLayerRes>()
        .expect("log_layer_res is None");
    log_layer_res.worker_guard_vec.push(worker_guard);

    Some(file_fmt_layer.boxed())
}
