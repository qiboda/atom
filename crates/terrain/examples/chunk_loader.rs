use atom_core::logger::atom_log_plugin;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::prelude::*;
use terrain::{
    chunks::loader::{
        TerrainChunkLoadMsg, TerrainChunkUnloadMsg, TerrainLoadedChunks,
        observer::{TerrainObserver, TerrainObserverConfig},
    },
    terrain::{TerrainPlugin, TerrainSystems, setting::*},
};
use tracing::Level;

fn main() {
    let mut app = App::new();
    app
        // 使用自定义的带文件输出的 LogPlugin
        .add_plugins(DefaultPlugins.set(atom_log_plugin(
            "info,terrain=trace".to_owned(),
            Level::TRACE,
            "terrain.log",
        )))
        .add_plugins(TerrainPlugin)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, log_chunk_events.after(TerrainSystems::ChunkLoader));

    app.run();
}

fn setup(mut commands: Commands, mut terrain_setting: ResMut<TerrainSetting>) {
    // 配置摄像机
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 1000.0,
            ..Default::default()
        }),
        Transform::from_xyz(50.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        TerrainObserver,
        TerrainObserverConfig {
            terrain_load_radius: Some(30),
            terrain_height_range: -40..=40,
        },
        FreeCamera {
            ..Default::default()
        },
    ));

    // 配置 Clipmap
    terrain_setting.clipmap_config = ClipmapConfig {
        lod_count: 3,
        lod0_radius: 4,
    };
    terrain_setting.size_setting.height_range = -40..=40;

    info!("地形系统启动完成");
}

// 日志系统：记录 chunk 加载/卸载事件
fn log_chunk_events(
    mut load_events: MessageReader<TerrainChunkLoadMsg>,
    mut unload_events: MessageReader<TerrainChunkUnloadMsg>,
    loaded_chunks: Res<TerrainLoadedChunks>,
    terrain_setting: Res<TerrainSetting>,
    mut gizmos: Gizmos,
) {
    for event in load_events.read() {
        info!("加载 Chunk: {:?}, LOD: {}", event.coord, event.lod);
    }

    for event in unload_events.read() {
        info!("卸载 Chunk: {:?}, LOD: {}", event.coord, event.lod);
    }

    gizmos.axes(Transform::IDENTITY, 10.0);

    // loaded_chunks
    for (lod, chunks) in loaded_chunks.iter() {
        for (coord, _entity) in chunks.iter() {
            if coord.y() != 0 {
                continue;
            }
            let lod_chunk_size = terrain_setting.get_chunk_size_by_lod(*lod);

            let mut location =
                coord.to_world_pos(lod_chunk_size) + Vec3::splat(lod_chunk_size / 2.0);
            location.y = 0.0;
            let transform = Transform::from_translation(location).with_scale(Vec3::new(
                lod_chunk_size * 0.9,
                1.0,
                lod_chunk_size * 0.9,
            ));
            match lod {
                0 => gizmos.cube(transform, Color::linear_rgb(0.0, 1.0, 0.0)),
                1 => gizmos.cube(transform, Color::linear_rgb(0.0, 0.0, 1.0)),
                2 => gizmos.cube(transform, Color::linear_rgb(1.0, 1.0, 0.0)),
                3 => gizmos.cube(transform, Color::linear_rgb(1.0, 0.0, 1.0)),
                _ => {}
            }
        }
    }
}
