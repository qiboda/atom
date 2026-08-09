//! 地形动态加载系统。
//!
//! 每帧根据观察者（`TerrainObserver`）位置加载/卸载 chunk，通过消息队列驱动 GPU compute 管线。

use bevy::prelude::*;

use crate::{
    chunk::{ChunkLoadMsg, ChunkUnloadMsg, TerrainChunkCoord, TerrainLoadedChunks},
    setting::TerrainSetting,
};

/// 观察者标记组件，挂载到玩家相机等需要加载地形 chunk 的实体上
#[derive(Component, Default)]
#[require(Transform)]
pub struct TerrainObserver;

/// 观察者配置，控制加载半径与高度范围
#[derive(Component)]
#[require(TerrainObserver)]
pub struct TerrainObserverConfig {
    /// 以 chunk 为单位的水平加载半径
    pub load_radius: u32,
    /// 垂直加载范围（相对于观察者 chunk Y）
    pub height_range: std::ops::RangeInclusive<i32>,
    /// 卸载宽松边界（chunk 单位），防止边界抖动
    pub margin: u32,
}

impl Default for TerrainObserverConfig {
    fn default() -> Self {
        Self {
            load_radius: 3,
            height_range: -2..=2,
            margin: 1,
        }
    }
}

/// 每帧根据观察者位置加载/卸载 chunk
pub fn update_grid_chunks(
    observers: Query<(&GlobalTransform, &TerrainObserverConfig), With<TerrainObserver>>,
    terrain_setting: Res<TerrainSetting>,
    loaded: Res<TerrainLoadedChunks>,
    mut load_tx: MessageWriter<ChunkLoadMsg>,
    mut unload_tx: MessageWriter<ChunkUnloadMsg>,
) {
    let chunk_size = terrain_setting.chunk_size();
    let mut keep: Vec<TerrainChunkCoord> = Vec::new();

    for (transform, config) in observers.iter() {
        let center = transform.translation();
        let center_chunk = TerrainChunkCoord::from_world(center, chunk_size);
        let r = config.load_radius as i32;
        let margin = config.margin as i32;
        let unload_r = r + margin;

        for y in center_chunk.0.y + *config.height_range.start()
            ..=center_chunk.0.y + *config.height_range.end()
        {
            for x in center_chunk.0.x - unload_r..=center_chunk.0.x + unload_r {
                for z in center_chunk.0.z - unload_r..=center_chunk.0.z + unload_r {
                    let coord = TerrainChunkCoord::new(x, y, z);
                    keep.push(coord);
                    if !loaded.contains(&coord) && x.abs() <= r && z.abs() <= r {
                        load_tx.write(ChunkLoadMsg { coord });
                    }
                }
            }
        }
    }

    let to_unload: Vec<TerrainChunkCoord> = loaded
        .iter()
        .filter(|(c, _)| !keep.contains(c))
        .map(|(c, _)| *c)
        .collect();

    // 只发卸载消息，不 despawn — 实际清理由 handle_unload_requests 完成
    for coord in to_unload {
        unload_tx.write(ChunkUnloadMsg { coord });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::TerrainChunk;
    use bevy::{MinimalPlugins, ecs::message::Messages};

    fn coord(x: i32, y: i32, z: i32) -> TerrainChunkCoord {
        TerrainChunkCoord::new(x, y, z)
    }

    fn spawn_observer(
        app: &mut App,
        load_radius: u32,
        height_range: std::ops::RangeInclusive<i32>,
        margin: u32,
        pos: Vec3,
    ) {
        app.world_mut().spawn((
            TerrainObserver,
            TerrainObserverConfig {
                load_radius,
                height_range,
                margin,
            },
            Transform::from_translation(pos),
            GlobalTransform::from_translation(pos),
        ));
    }

    fn loaded_coords(app: &App) -> Vec<TerrainChunkCoord> {
        app.world()
            .resource::<Messages<ChunkLoadMsg>>()
            .iter_current_update_messages()
            .map(|m| m.coord)
            .collect()
    }

    fn unloaded_coords(app: &App) -> Vec<TerrainChunkCoord> {
        app.world()
            .resource::<Messages<ChunkUnloadMsg>>()
            .iter_current_update_messages()
            .map(|m| m.coord)
            .collect()
    }

    fn loader_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ChunkLoadMsg>();
        app.add_message::<ChunkUnloadMsg>();
        app.insert_resource(TerrainSetting::default());
        app.init_resource::<TerrainLoadedChunks>();
        app.add_systems(Update, update_grid_chunks);
        app
    }

    #[test]
    fn observer_config_default_values() {
        let c = TerrainObserverConfig::default();
        assert_eq!(c.load_radius, 3);
        assert_eq!(c.height_range, -2..=2);
        assert_eq!(c.margin, 1);
    }

    fn sorted(mut v: Vec<TerrainChunkCoord>) -> Vec<TerrainChunkCoord> {
        v.sort_by_key(|c| (c.0.x, c.0.y, c.0.z));
        v
    }

    #[test]
    fn loads_chunks_within_radius_around_observer() {
        let mut app = loader_app();
        spawn_observer(&mut app, 1, -1..=1, 0, Vec3::new(7.5, 0.0, 7.5));

        app.update();

        // center_chunk = (0,0,0)；r=1 → x,z ∈ [-1,1]，y ∈ [-1,1] → 3³ = 27
        let loads = loaded_coords(&app);
        assert_eq!(loads.len(), 27);
        for c in &loads {
            assert!(
                (-1..=1).contains(&c.0.x) && (-1..=1).contains(&c.0.y) && (-1..=1).contains(&c.0.z),
                "越界坐标 {c:?}"
            );
        }
        assert!(unloaded_coords(&app).is_empty());
    }

    #[test]
    fn does_not_reload_already_loaded_chunk() {
        let mut app = loader_app();
        let existing = app.world_mut().spawn(TerrainChunk).id();
        app.world_mut()
            .resource_mut::<TerrainLoadedChunks>()
            .insert(coord(0, 0, 0), existing);
        spawn_observer(&mut app, 1, -1..=1, 0, Vec3::ZERO);

        app.update();

        assert_eq!(
            loaded_coords(&app).len(),
            26,
            "已加载的 (0,0,0) 不再重复加载"
        );
        assert!(!loaded_coords(&app).contains(&coord(0, 0, 0)));
    }

    #[test]
    fn unloads_chunks_outside_keep_region() {
        let mut app = loader_app();
        {
            let e1 = app.world_mut().spawn(TerrainChunk).id();
            let e2 = app.world_mut().spawn(TerrainChunk).id();
            let mut loaded = app.world_mut().resource_mut::<TerrainLoadedChunks>();
            loaded.insert(coord(5, 0, 5), e1);
            loaded.insert(coord(2, 0, 0), e2);
        }
        spawn_observer(&mut app, 1, -1..=1, 1, Vec3::ZERO);

        app.update();

        // margin=1 → keep 区域 x,z ∈ [-2,2]；(2,0,0) 在 keep 内不卸载
        let unloads = unloaded_coords(&app);
        assert_eq!(unloads, vec![coord(5, 0, 5)]);
        // (2,0,0) 在 keep 内 → 不加载也不卸载
        let loads = loaded_coords(&app);
        assert!(!loads.contains(&coord(2, 0, 0)));
    }

    #[test]
    fn no_observer_sends_no_messages() {
        let mut app = loader_app();

        app.update();

        assert!(loaded_coords(&app).is_empty());
        assert!(unloaded_coords(&app).is_empty());
    }

    #[test]
    fn load_condition_uses_absolute_chunk_coords() {
        let mut app = loader_app();
        spawn_observer(&mut app, 1, -1..=1, 0, Vec3::new(-7.5, 0.0, -7.5));

        app.update();

        // center_chunk = (-1,0,-1)：keep 区域 x,z ∈ [-2,0]；加载条件为 |x|<=1,|z|<=1
        // （对绝对 chunk 坐标求绝对值）→ 实际加载 x,z ∈ {-1,0} → 2*2*3 = 12
        let loads = sorted(loaded_coords(&app));
        assert_eq!(loads.len(), 12);
        for c in &loads {
            assert!(
                (-1..=1).contains(&c.0.x) && (-1..=1).contains(&c.0.z),
                "加载坐标 {c:?} 越界"
            );
            assert!((-2..=0).contains(&c.0.x) && (-2..=0).contains(&c.0.z));
        }
        assert!(loads.contains(&coord(-1, 0, 0)));
        assert!(!loads.contains(&coord(-2, 0, -2)));
    }
}
