use atom_core::logger::atom_log_plugin;
use atom_core::paths::ProjectPaths;
use atom_renderdoc::RenderDocPlugin;
use atom_terrain::{
    chunks::loader::{
        TerrainLoadedChunks,
        observer::{TerrainObserver, TerrainObserverConfig},
    },
    terrain::{TerrainPlugin, TerrainSystems, setting::*},
};
use bevy::{
    diagnostic::{
        FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    pbr::wireframe::WireframeConfig,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};
use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use tracing::Level;

fn main() {
    let mut app = App::new();
    app
        // 使用自定义的带文件输出的 LogPlugin
        .add_plugins(
            DefaultPlugins
                .set(atom_log_plugin(
                    "info,atom_terrain=trace".to_owned(),
                    Level::DEBUG,
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
        // 诊断用途
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(RemotePlugin::default())
        .add_plugins(RemoteHttpPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(RenderDocPlugin)
        .add_plugins(WireframePlugin::default())
        // .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            gizmos_loaded_chunk.after(TerrainSystems::ChunkLoader),
        );

    // 摄像机插件
    app.add_plugins(NoCameraPlayerPlugin);

    // 地形插件
    app.add_plugins(TerrainPlugin { debug: false });
    app.insert_resource(TerrainSetting {
        size_setting: TerrainSizeSetting {
            height_range: -1..=1,
            ..Default::default()
        },
        ..Default::default()
    });

    // 全局显示线框
    app.insert_resource(WireframeConfig {
        global: true,
        ..Default::default()
    });

    app.run();
}

fn setup(mut commands: Commands) {
    // 配置摄像机
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 1000.0,
            ..Default::default()
        }),
        Transform::from_xyz(-10.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        TerrainObserver,
        TerrainObserverConfig {
            terrain_load_radius: 0,
            terrain_height_range: -3..=3,
            margin: 1,
        },
        FlyCam,
        // FreeCamera {
        //     ..Default::default()
        // },
    ));

    info!("地形系统启动完成");
}

fn gizmos_loaded_chunk(
    loaded_chunks: Res<TerrainLoadedChunks>,
    terrain_setting: Res<TerrainSetting>,
    mut gizmos: Gizmos,
) {
    // info!(
    //     "mesh_compute_vertices_shader id: {:?}",
    //     mesh_compute_shaders.mesh_compute_vertices_shader.id()
    // );

    // for event in load_events.read() {
    //     info!("加载 Chunk: {:?}", event.coord);
    // }

    // for event in unload_events.read() {
    //     info!("卸载 Chunk: {:?}", event.coord);
    // }

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
