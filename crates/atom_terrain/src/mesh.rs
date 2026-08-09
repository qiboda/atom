//! Chunk mesh 数据跨线程收发。
//!
//! 定义从渲染世界回传到主世界的 mesh 数据结构与 channel 收发器。

use bevy::{prelude::*, render::render_resource::Face};
use crossbeam::channel::{Receiver, Sender};

use crate::{
    chunk::{ChunkLoadMsg, ChunkUnloadMsg, TerrainChunk, TerrainChunkCoord, TerrainLoadedChunks},
    compute::sync::{ChunkProcessRequest, TerrainChunkProcessSender},
    debug::TerrainDebugConfig,
    setting::TerrainSetting,
};

/// 从 GPU compute 管线回传到主世界的 chunk mesh 数据
pub struct TerrainChunkMeshData {
    /// 生成的网格数据
    pub mesh: Mesh,
    /// chunk 世界坐标偏移
    pub translation: Vec3,
}

/// 主世界端的 mesh 接收器，包装 crossbeam channel `Receiver`
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshReceiver(pub Receiver<TerrainChunkMeshData>);

/// 渲染世界端的 mesh 发送器，包装 crossbeam channel `Sender`
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshSender(pub Sender<TerrainChunkMeshData>);

/// Chunk 网格化状态机
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainChunkMeshingState {
    /// 空闲，等待加载
    #[default]
    Idle,
    /// GPU compute 处理中
    Meshing,
    /// compute 完成，等待读回
    Done,
}

/// 接收主世界的 chunk 加载消息，首次加载时创建实体，注册并通过 channel 发送到渲染世界。
pub fn handle_load_requests(
    mut commands: Commands,
    mut reader: MessageReader<ChunkLoadMsg>,
    setting: Res<TerrainSetting>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
    sender: Res<TerrainChunkProcessSender>,
) {
    let chunk_size = setting.chunk_size();
    for msg in reader.read() {
        if loaded_chunks.contains(&msg.coord) {
            // 已在队列中，跳过（可能是前一帧刚创建的）
            continue;
        }
        let world_pos = msg.coord.to_world(chunk_size);
        let entity = commands
            .spawn((
                TerrainChunk,
                TerrainChunkCoord(msg.coord.0),
                TerrainChunkMeshingState::Meshing,
                Transform::IDENTITY, // 顶点已是世界坐标，父 entity 不加偏移
                Visibility::default(),
            ))
            .id();
        loaded_chunks.insert(msg.coord, entity);
        let _ = sender.send(ChunkProcessRequest::Load {
            entity,
            world_min: world_pos,
        });
    }
}

/// 接收卸载消息: despawn chunk entity（含 children mesh），清理注册表，通知渲染世界释放 GPU buffer。
pub fn handle_unload_requests(
    mut commands: Commands,
    mut reader: MessageReader<ChunkUnloadMsg>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
    sender: Res<TerrainChunkProcessSender>,
) {
    for msg in reader.read() {
        if let Some(entity) = loaded_chunks.remove(&msg.coord) {
            commands.entity(entity).despawn();
            let _ = sender.send(ChunkProcessRequest::Unload { entity });
        }
    }
}

/// 接收渲染世界发来的 mesh，spawn 为 chunk entity 的子实体
pub fn handle_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshReceiver>,
    setting: Res<TerrainSetting>,
    loaded_chunks: Res<TerrainLoadedChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    while let Ok(data) = receiver.try_recv() {
        let mesh = meshes.add(data.mesh);
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.75, 0.8),
            perceptual_roughness: 0.3,
            ..default()
        });
        // 通过 translation 反查 TerrainChunkCoord，找到父 chunk entity
        let coord = TerrainChunkCoord::from_world(data.translation, setting.chunk_size());
        if let Some(chunk_entity) = loaded_chunks.get(&coord) {
            commands.entity(chunk_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::IDENTITY, // 顶点已由 shader 转为世界坐标
                    Visibility::default(),
                ));
            });
            commands
                .entity(chunk_entity)
                .insert(TerrainChunkMeshingState::Done);
        }
    }
}

/// 全局 mesh 标记组件（Phase 3 global pipeline）。
#[derive(Component)]
pub struct GlobalTerrainMesh;

/// 全局地形 material handle（Phase 3），供 debug toggle 更新 cull_mode。
#[derive(Resource, Clone, Debug, Default)]
pub struct GlobalTerrainMaterial(pub Option<Handle<StandardMaterial>>);

/// 接收渲染世界发来的全局 mesh（Phase 3），直接 spawn 实体。
///
/// 与 `handle_mesh_data` 不同，不依赖 TerrainLoadedChunks，
/// mesh 直接独立 spawn（顶点已为世界坐标）。
pub fn handle_global_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshReceiver>,
    debug_config: Res<TerrainDebugConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_entity: Local<Option<Entity>>,
    mut global_mat: ResMut<GlobalTerrainMaterial>,
) {
    while let Ok(data) = receiver.try_recv() {
        let mesh = meshes.add(data.mesh);
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.75, 0.8),
            perceptual_roughness: 0.3,
            cull_mode: if debug_config.double_sided {
                None
            } else {
                Some(Face::Back)
            },
            ..default()
        });

        // 存储 material handle，供 debug toggle 更新
        global_mat.0 = Some(mat.clone());

        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::IDENTITY,
                Visibility::default(),
                GlobalTerrainMesh,
            ))
            .id();

        // 新 mesh 就绪后再 despawn 旧的 → 避免空窗期
        if let Some(old) = mesh_entity.replace(entity) {
            commands.entity(old).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::TerrainChunk;
    use crate::compute::sync::ChunkProcessRequest;
    use bevy::MinimalPlugins;

    fn coord(x: i32, y: i32, z: i32) -> TerrainChunkCoord {
        TerrainChunkCoord::new(x, y, z)
    }

    fn test_mesh() -> Mesh {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        );
        mesh
    }

    // ── 状态枚举 / 资源 ──

    #[test]
    fn meshing_state_default_is_idle() {
        assert_eq!(
            TerrainChunkMeshingState::default(),
            TerrainChunkMeshingState::Idle
        );
        assert_ne!(
            TerrainChunkMeshingState::Idle,
            TerrainChunkMeshingState::Meshing
        );
        assert_ne!(
            TerrainChunkMeshingState::Meshing,
            TerrainChunkMeshingState::Done
        );
    }

    #[test]
    fn global_terrain_material_default_is_none() {
        let m = GlobalTerrainMaterial::default();
        assert!(m.0.is_none());
    }

    // ── handle_load_requests ──

    fn load_app() -> (App, crossbeam::channel::Receiver<ChunkProcessRequest>) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ChunkLoadMsg>();
        app.insert_resource(TerrainSetting::default());
        app.init_resource::<TerrainLoadedChunks>();
        let (tx, rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkProcessSender(tx));
        app.add_systems(Update, handle_load_requests);
        (app, rx)
    }

    #[test]
    fn load_request_spawns_chunk_entity_and_sends_request() {
        let (mut app, rx) = load_app();

        // 预加载 (0,0,0) → 该消息应被跳过
        let existing = app.world_mut().spawn(TerrainChunk).id();
        app.world_mut()
            .resource_mut::<TerrainLoadedChunks>()
            .insert(coord(0, 0, 0), existing);

        app.world_mut().write_message(ChunkLoadMsg {
            coord: coord(0, 0, 0),
        });
        app.world_mut().write_message(ChunkLoadMsg {
            coord: coord(1, 0, 0),
        });

        app.update();

        // 只为 (1,0,0) 创建了 chunk entity，状态为 Meshing
        let mut q = app
            .world_mut()
            .query::<(Entity, &TerrainChunkCoord, &TerrainChunkMeshingState)>();
        let found: Vec<_> = q.iter(app.world()).map(|(e, &c, &s)| (e, c, s)).collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, coord(1, 0, 0));
        assert_eq!(found[0].2, TerrainChunkMeshingState::Meshing);
        let spawned = found[0].0;

        // 渲染世界收到了 Load 请求，world_min 为 chunk 左下角
        match rx.try_recv().expect("应收到 Load 请求") {
            ChunkProcessRequest::Load { entity, world_min } => {
                assert_eq!(entity, spawned, "请求 entity 应为新 chunk 实体");
                assert_eq!(world_min, coord(1, 0, 0).to_world(15.0));
            }
            _ => panic!("expected ChunkProcessRequest::Load"),
        }
        assert!(rx.try_recv().is_err(), "只有一个 Load 请求");
    }

    #[test]
    fn load_request_duplicate_coord_is_skipped() {
        let (mut app, rx) = load_app();

        app.world_mut().write_message(ChunkLoadMsg {
            coord: coord(2, 0, 0),
        });
        app.world_mut().write_message(ChunkLoadMsg {
            coord: coord(2, 0, 0),
        });

        app.update();

        let mut q = app.world_mut().query::<&TerrainChunkCoord>();
        assert_eq!(q.iter(app.world()).count(), 1, "重复坐标只创建一次");

        match rx.try_recv().expect("应收到一个 Load 请求") {
            ChunkProcessRequest::Load { world_min, .. } => {
                assert_eq!(world_min, coord(2, 0, 0).to_world(15.0));
            }
            _ => panic!("expected ChunkProcessRequest::Load"),
        }
        assert!(rx.try_recv().is_err());
    }

    // ── handle_unload_requests ──

    fn unload_app() -> (App, crossbeam::channel::Receiver<ChunkProcessRequest>) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ChunkUnloadMsg>();
        app.init_resource::<TerrainLoadedChunks>();
        let (tx, rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkProcessSender(tx));
        app.add_systems(Update, handle_unload_requests);
        (app, rx)
    }

    #[test]
    fn unload_request_despawns_entity_and_notifies_render_world() {
        let (mut app, rx) = unload_app();

        let e1 = app.world_mut().spawn(TerrainChunk).id();
        let e2 = app.world_mut().spawn(TerrainChunk).id();
        {
            let mut loaded = app.world_mut().resource_mut::<TerrainLoadedChunks>();
            loaded.insert(coord(0, 0, 0), e1);
            loaded.insert(coord(1, 0, 0), e2);
        }

        app.world_mut().write_message(ChunkUnloadMsg {
            coord: coord(0, 0, 0),
        });
        // 未注册的坐标 → 被忽略
        app.world_mut().write_message(ChunkUnloadMsg {
            coord: coord(9, 9, 9),
        });

        app.update();

        assert!(app.world().get_entity(e1).is_err(), "e1 应被 despawn");
        assert!(app.world().get_entity(e2).is_ok(), "e2 不应被 despawn");
        let loaded = app.world().resource::<TerrainLoadedChunks>();
        assert!(!loaded.contains(&coord(0, 0, 0)));
        assert!(loaded.contains(&coord(1, 0, 0)));

        match rx.try_recv().expect("应收到 Unload 请求") {
            ChunkProcessRequest::Unload { entity } => assert_eq!(entity, e1),
            _ => panic!("expected ChunkProcessRequest::Unload"),
        }
        assert!(rx.try_recv().is_err(), "只对已注册坐标发 Unload");
    }

    #[test]
    fn unload_unknown_coord_does_nothing() {
        let (mut app, rx) = unload_app();

        app.world_mut().write_message(ChunkUnloadMsg {
            coord: coord(0, 0, 0),
        });
        app.update();

        let loaded = app.world().resource::<TerrainLoadedChunks>();
        assert_eq!(loaded.len(), 0);
        assert!(rx.try_recv().is_err());
    }

    // ── handle_mesh_data ──

    fn mesh_data_app() -> (App, crossbeam::channel::Sender<TerrainChunkMeshData>) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TerrainSetting::default());
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<TerrainLoadedChunks>();
        let (tx, rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkMeshReceiver(rx));
        app.add_systems(Update, handle_mesh_data);
        (app, tx)
    }

    #[test]
    fn mesh_data_spawns_child_mesh_and_marks_chunk_done() {
        let (mut app, tx) = mesh_data_app();

        let chunk = app
            .world_mut()
            .spawn((TerrainChunk, TerrainChunkCoord(bevy::math::IVec3::ZERO)))
            .id();
        app.world_mut()
            .resource_mut::<TerrainLoadedChunks>()
            .insert(coord(0, 0, 0), chunk);

        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::ZERO,
        })
        .expect("send mesh data");

        app.update();

        assert_eq!(
            *app.world()
                .entity(chunk)
                .get::<TerrainChunkMeshingState>()
                .expect("chunk 应有 meshing state"),
            TerrainChunkMeshingState::Done
        );

        // chunk 下应挂了一个带 Mesh3d 的子实体
        let mut q = app.world_mut().query::<(&ChildOf, &Mesh3d)>();
        let children: Vec<_> = q
            .iter(app.world())
            .filter(|(c, _)| c.parent() == chunk)
            .collect();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn mesh_data_without_matching_chunk_is_ignored() {
        let (mut app, tx) = mesh_data_app();

        // translation 不在任何已加载 chunk 内
        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::new(999.0, 0.0, 999.0),
        })
        .expect("send mesh data");

        app.update();

        let mut q = app.world_mut().query::<&ChildOf>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "无匹配 chunk 时不应 spawn 子实体"
        );
    }

    // ── handle_global_mesh_data ──

    fn global_mesh_app(
        double_sided: bool,
    ) -> (App, crossbeam::channel::Sender<TerrainChunkMeshData>) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(TerrainDebugConfig {
            double_sided,
            ..Default::default()
        });
        app.init_resource::<GlobalTerrainMaterial>();
        let (tx, rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkMeshReceiver(rx));
        app.add_systems(Update, handle_global_mesh_data);
        (app, tx)
    }

    #[test]
    fn global_mesh_replaces_previous_entity() {
        let (mut app, tx) = global_mesh_app(true);

        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::ZERO,
        })
        .expect("send");
        app.update();

        // 第二帧新 mesh 就绪 → 旧实体 despawn，新实体存在
        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::ZERO,
        })
        .expect("send");
        app.update();

        let mut q = app.world_mut().query::<(Entity, &GlobalTerrainMesh)>();
        let live: Vec<_> = q.iter(app.world()).map(|(e, _)| e).collect();
        assert_eq!(live.len(), 1, "始终只有一个 global mesh");

        // material handle 已更新，double_sided=true → cull_mode None
        let handle = app
            .world()
            .resource::<GlobalTerrainMaterial>()
            .0
            .as_ref()
            .expect("material handle 已写入");
        let mat = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(handle)
            .expect("material 存在");
        assert_eq!(mat.cull_mode, None);
    }

    #[test]
    fn global_mesh_keeps_one_entity_across_two_updates() {
        // 覆盖 mesh_entity Local 的 replace 分支：第二次消息到来时旧实体被 despawn
        let (mut app, tx) = global_mesh_app(true);

        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::ZERO,
        })
        .expect("send");
        app.update();

        let first = {
            let mut q = app.world_mut().query::<(Entity, &GlobalTerrainMesh)>();
            q.iter(app.world()).map(|(e, _)| e).next().expect("有实体")
        };

        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::ZERO,
        })
        .expect("send");
        app.update();

        assert!(app.world().get_entity(first).is_err(), "旧实体应被 despawn");
        let mut q = app.world_mut().query::<&GlobalTerrainMesh>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn global_mesh_cull_back_when_not_double_sided() {
        let (mut app, tx) = global_mesh_app(false);

        tx.send(TerrainChunkMeshData {
            mesh: test_mesh(),
            translation: Vec3::ZERO,
        })
        .expect("send");
        app.update();

        let handle = app
            .world()
            .resource::<GlobalTerrainMaterial>()
            .0
            .as_ref()
            .expect("material handle 已写入");
        let mat = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(handle)
            .expect("material 存在");
        assert_eq!(mat.cull_mode, Some(Face::Back));
    }
}
