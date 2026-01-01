use atom_core::logger::atom_log_plugin;
use atom_core::paths::ProjectPaths;
use atom_pqef::QuadricPlugin;
use atom_shader_lib::AtomShaderLibPluginGroups;
use atom_terrain::{
    chunks::loader::{
        TerrainChunkLoadMsg, TerrainChunkUnloadMsg, TerrainLoadedChunks,
        observer::{TerrainObserver, TerrainObserverConfig},
    },
    terrain::{TerrainPlugin, TerrainSystems, setting::*},
};
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::prelude::*;
use tracing::Level;

fn main() {
    let mut app = App::new();
    app
        // 使用自定义的带文件输出的 LogPlugin
        .add_plugins(
            DefaultPlugins
                .set(atom_log_plugin(
                    "info,terrain=trace".to_owned(),
                    Level::TRACE,
                    "terrain",
                ))
                .set(AssetPlugin {
                    file_path: ProjectPaths::assets_path().to_string_lossy().into(),
                    processed_file_path: ProjectPaths::processed_assets_path()
                        .to_string_lossy()
                        .into(),
                    ..default()
                }),
        )
        .add_plugins(QuadricPlugin)
        .add_plugins(AtomShaderLibPluginGroups)
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
            terrain_load_radius: 3,
            terrain_height_range: -2..=2,
        },
        FreeCamera {
            ..Default::default()
        },
    ));

    terrain_setting.size_setting.height_range = -2..=2;

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
        info!("加载 Chunk: {:?}", event.coord);
    }

    for event in unload_events.read() {
        info!("卸载 Chunk: {:?}", event.coord);
    }

    gizmos.axes(Transform::IDENTITY, 10.0);

    let chunk_size = terrain_setting.chunk_setting.get_chunk_size();

    for (coord, _entity) in loaded_chunks.iter() {
        if coord.y() != 0 {
            continue;
        }

        let mut location = coord.to_world_pos(chunk_size) + Vec3::splat(chunk_size / 2.0);
        location.y = 0.0;
        let transform = Transform::from_translation(location).with_scale(Vec3::new(
            chunk_size * 0.9,
            1.0,
            chunk_size * 0.9,
        ));
        gizmos.cube(transform, Color::linear_rgb(0.0, 1.0, 0.0));
    }
}
