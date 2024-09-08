use bevy::{log::BoxedLayer, prelude::*};
use project::project_saved_root_path;
use tracing_subscriber::{filter::LevelFilter, Layer};

use crate::{BoxedLayerFn, LogLayerGuardRes};

pub fn file_layer(app: &mut App, filename: &str) -> Option<BoxedLayer> {
    let saved_path = project_saved_root_path();

    let file_appender = tracing_appender::rolling::daily(saved_path.join("logs"), filename); // This should be user configurable
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
    let file_fmt_layer = tracing_subscriber::fmt::Layer::default()
        .with_ansi(false) // disable terminal color escape sequences
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .with_writer(non_blocking)
        .with_filter(LevelFilter::TRACE);

    let mut log_layer_guard_res = app
        .world_mut()
        .get_resource_mut::<LogLayerGuardRes>()
        .expect("log_layer_res is None");
    log_layer_guard_res.worker_guard_vec.push(worker_guard);

    Some(file_fmt_layer.boxed())
}

pub fn file_layer_with_filename(filename: String) -> BoxedLayerFn {
    Box::new(move |app: &mut App| file_layer(app, filename.as_str()))
}
