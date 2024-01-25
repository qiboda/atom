use bevy::{
    prelude::*,
    utils::tracing::{self},
};

use project::project_saved_root_path;
use terrain_player_client::trace::{terrain_tracing::TerrainLayer, TERRAIN_TRACE_TARGET};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_log::{log::Level, LogTracer};
#[cfg(feature = "tracing-chrome")]
use tracing_subscriber::fmt::{format::DefaultFields, FormattedFields};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

#[derive(Resource, Debug)]
pub struct FileLogRes {
    pub worker_guard: tracing_appender::non_blocking::WorkerGuard,
    pub terrain_workder_guard: tracing_appender::non_blocking::WorkerGuard,
}

pub struct CustomLogPlugin {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: Level,
}

impl Default for CustomLogPlugin {
    fn default() -> Self {
        Self {
            filter: "info".to_string(),
            level: Level::Info,
        }
    }
}

impl Plugin for CustomLogPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "trace")]
        {
            let old_handler = panic::take_hook();
            panic::set_hook(Box::new(move |infos| {
                println!("{}", tracing_error::SpanTrace::capture());
                old_handler(infos);
            }));
        }

        let finished_subscriber;
        let default_filter = { format!("{},{},terrain_trace=trace", self.level, self.filter) };
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&default_filter))
            .unwrap();
        let subscriber = Registry::default().with(filter_layer);

        #[cfg(feature = "trace")]
        let subscriber = subscriber.with(tracing_error::ErrorLayer::default());

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
        {
            #[cfg(feature = "trace_chrome")]
            let chrome_layer = {
                let mut layer = tracing_chrome::ChromeLayerBuilder::new();
                if let Ok(path) = std::env::var("TRACE_CHROME") {
                    layer = layer.file(path);
                }
                let (chrome_layer, guard) = layer
                    .name_fn(Box::new(|event_or_span| match event_or_span {
                        tracing_chrome::EventOrSpan::Event(event) => event.metadata().name().into(),
                        tracing_chrome::EventOrSpan::Span(span) => {
                            if let Some(fields) =
                                span.extensions().get::<FormattedFields<DefaultFields>>()
                            {
                                format!("{}: {}", span.metadata().name(), fields.fields.as_str())
                            } else {
                                span.metadata().name().into()
                            }
                        }
                    }))
                    .build();
                app.world.insert_non_send_resource(guard);
                chrome_layer
            };

            #[cfg(feature = "trace_tracy")]
            let tracy_layer = tracing_tracy::TracyLayer::new();

            let std_filter = EnvFilter::new("warning");
            let fmt_layer = tracing_subscriber::fmt::Layer::default()
                .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
                .with_writer(std::io::stderr)
                .with_filter(std_filter);

            // bevy_render::renderer logs a `tracy.frame_mark` event every frame
            // at Level::INFO. Formatted logs should omit it.
            #[cfg(feature = "trace_tracy")]
            let fmt_layer =
                fmt_layer.with_filter(tracing_subscriber::filter::FilterFn::new(|meta| {
                    meta.fields().field("tracy.frame_mark").is_none()
                }));

            let saved_path = project_saved_root_path();

            let file_filter = EnvFilter::new("info");

            let file_appender = tracing_appender::rolling::daily(saved_path.join("logs"), "log"); // This should be user configurable
            let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
            let file_fmt_layer = tracing_subscriber::fmt::Layer::default()
                .with_ansi(false) // disable terminal color escape sequences
                .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
                .with_writer(non_blocking)
                .with_filter(file_filter);

            let terrain_filter = EnvFilter::new(TERRAIN_TRACE_TARGET.to_owned() + "=trace");

            let trace_path = saved_path.join("trace");
            let appender = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_prefix("terrain_trace")
                .build(trace_path)
                .unwrap();
            let (non_blocking, terrain_worker_guard) = tracing_appender::non_blocking(appender);

            let terrain_trace_fmt = TerrainLayer::new()
                .with_pretty(false)
                .with_writer(non_blocking)
                .with_filter(terrain_filter);

            app.insert_resource(FileLogRes {
                worker_guard,
                terrain_workder_guard: terrain_worker_guard,
            }); // have to keep this from being dropped

            let subscriber = subscriber
                .with(fmt_layer)
                .with(file_fmt_layer)
                .with(terrain_trace_fmt);

            #[cfg(feature = "trace_chrome")]
            let subscriber = subscriber.with(chrome_layer);
            #[cfg(feature = "trace_tracy")]
            let subscriber = subscriber.with(tracy_layer);

            finished_subscriber = subscriber;
        }

        #[cfg(target_arch = "wasm32")]
        {
            console_error_panic_hook::set_once();
            finished_subscriber = subscriber.with(tracing_wasm::WASMLayer::new(
                tracing_wasm::WASMLayerConfig::default(),
            ));
        }

        #[cfg(target_os = "android")]
        {
            finished_subscriber = subscriber.with(android_tracing::AndroidLayer::default());
        }

        let logger_already_set = LogTracer::init().is_err();
        let subscriber_already_set =
            tracing::subscriber::set_global_default(finished_subscriber).is_err();

        match (logger_already_set, subscriber_already_set) {
            (true, true) => warn!(
                "Could not set global logger and tracing subscriber as they are already set. Consider disabling LogPlugin."
            ),
            (true, _) => warn!("Could not set global logger as it is already set. Consider disabling LogPlugin."),
            (_, true) => warn!("Could not set global tracing subscriber as it is already set. Consider disabling LogPlugin."),
            _ => (),
        }
    }
}
