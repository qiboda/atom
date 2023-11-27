use bevy::prelude::*;
use bevy::utils::tracing::instrument::WithSubscriber;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;
use project::project_saved_root_path;

pub struct TerrainTracePlugin;

// 改为写在log初始化之前。
impl Plugin for TerrainTracePlugin {
    fn build(&self, app: &mut App) {

        let filter = EnvFilter::new(TERRAIN_TRACE_TARGET.to_owned() + "=trace");

        let trace_path = project_saved_root_path().join("trace");
        let appender = tracing_appender::rolling::never(trace_path, "trace");
        let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

        let fmt = tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_env_filter(filter);

        tracing_subscriber::registry().with_subscriber(fmt);
    }
}

pub const TERRAIN_TRACE_TARGET: &str = "terrain_trace";