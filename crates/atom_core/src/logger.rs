use std::sync::OnceLock;

use bevy::{
    log::{BoxedLayer, DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use tracing::{Level, level_filters::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::Layer;

use crate::paths::ProjectPaths;

#[derive(Resource)]
pub struct LogLayerGuardRes {
    pub worker_guard_vec: Vec<WorkerGuard>,
}

static LOG_FILENAME: OnceLock<String> = OnceLock::new();

/**
 * 无法封装进一个闭包中，来传递log的filename，因此目前使用全局静态变量。
 */
fn file_layer(app: &mut App) -> Option<BoxedLayer> {
    let saved_path = ProjectPaths::saved_path();

    app.insert_resource(LogLayerGuardRes {
        worker_guard_vec: Vec::new(),
    });

    let ts = time::OffsetDateTime::now_local()
        .expect("Failed to get local time")
        .format(
            &time::format_description::parse("[year]-[month]-[day]_[hour]-[minute]-[second]")
                .expect("Failed to parse time format description"),
        )
        .unwrap_or_default();

    let file_appender: tracing_appender::rolling::RollingFileAppender =
        tracing_appender::rolling::never(
            saved_path.join("logs"),
            format!(
                "{}-{}.log",
                LOG_FILENAME.get().expect("LOG_FILENAME is not set"),
                ts
            ),
        );
    // This should be user configurable
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

/**
 * `level` is the lowest global log level,
 * if you want to set specific module log level, please set it in `filter` string,
 * if you want to set global default log level, please set it in `filter` string like "info".
 */
pub fn atom_log_plugin(filter: String, level: Level, filename: &str) -> LogPlugin {
    LOG_FILENAME
        .set(filename.to_string())
        .expect("LOG_FILENAME already set - atom_log_plugin can only be called once");

    LogPlugin {
        filter: filter + "," + DEFAULT_FILTER,
        level,
        custom_layer: file_layer,
        ..default()
    }
}
