//! load/unload/reload 生命周期集成测试（启动 Bevy App + AssetServer 加载真实文件）。
//!
//! spec 依据：`.omo/plans/atom-data.md` §5（issue #4 Batch 2）——
//! - 验收「文档与测试覆盖 load/unload/reload 生命周期」「底层走 AssetServer + AssetEvent」
//! - §5.4「单表 get/load/unload/is_loaded 生命周期」
//! - B2-4「unload 后失效」
//!
//! 资产路径：Bevy 0.19 `FileAssetReader::get_base_path` 优先取 `CARGO_MANIFEST_DIR`
//! （crate 根 = `crates/atom_data`），AssetPlugin 默认 root `assets` →
//! `crates/atom_data/assets`（符号链接 → 仓库根 `assets/`），Q10 目录约定
//! `datatables/<表类型名>.json` 可解析。
//!
//! handle 存活说明：`load` 返回的强 handle 若立即释放，`Assets::track_assets`（PreUpdate）
//! 会在资产插入同帧将其移除——sync 系统读 `Assets` 取表时可能已不存在，测试对
//! `load`/`reload` 返回的 handle 保持存活至断言完成。

use atom_data::{DataAsset, DataRegistry, DataRegistryPlugin, DataTable};
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

/// 生命周期测试行类型：主索引 id。资产文件含额外字段 `kind`——serde 默认忽略未知字段。
#[derive(serde::Deserialize, DataAsset, Clone)]
#[index(key = "id")]
struct ItemConfig {
    id: i32,
    name: String,
}

/// 测试 App：MinimalPlugins + AssetPlugin（文件读取）+ DataRegistryPlugin。
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), DataRegistryPlugin));
    app.init_asset::<DataTable<ItemConfig>>();
    app.add_plugins(JsonAssetPlugin::<DataTable<ItemConfig>>::new(&["json"]));
    DataRegistryPlugin::register_table::<ItemConfig>(&mut app);
    app
}

/// 推进帧直到 ItemConfig 表进入 registry（AssetEvent 异步：任务池读取 → PreUpdate 事件
/// 处理 → Update sync）。帧上限防死循环。
///
/// 每帧 sleep 5ms：`app.update()` 本身不消耗真实时间，而 AssetServer 的文件读取在
/// `IoTaskPool` 上异步执行——nextest 并发运行时 IO 线程调度可能被延迟，纯帧推进会在
/// 几毫秒内耗尽帧上限而资产尚未到达。sleep 让异步 IO 有真实时间完成（flaky 修复）。
fn wait_until_loaded(app: &mut App, max_frames: usize) -> bool {
    for _ in 0..max_frames {
        app.update();
        if app
            .world()
            .resource::<DataRegistry>()
            .is_loaded::<ItemConfig>()
        {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    false
}

/// §5.3「加载后可查」+ 验收「load 生命周期」：load → AssetServer 异步加载 →
/// sync 系统入注册表 → 主索引命中真实文件数据。
#[test]
fn load_then_query_hits() {
    let mut app = test_app();
    let server = app.world().resource::<AssetServer>().clone();
    // handle 保持存活（见文件头说明）
    let _handle = app
        .world_mut()
        .resource_mut::<DataRegistry>()
        .load::<ItemConfig>(&server, "datatables/ItemConfig.json");

    assert!(
        wait_until_loaded(&mut app, 60),
        "60 帧内应完成 AssetServer 加载"
    );
    let registry = app.world().resource::<DataRegistry>();
    assert!(registry.is_loaded::<ItemConfig>());
    let row = registry.get::<ItemConfig>(&1).expect("加载后应命中");
    assert_eq!(row.id, 1);
    assert_eq!(row.name, "fireball");
}

/// B2-4「unload 后失效」+ 验收「unload 生命周期」：unload 后 is_loaded false、查询 miss。
#[test]
fn unload_then_query_misses() {
    let mut app = test_app();
    let server = app.world().resource::<AssetServer>().clone();
    let _handle = app
        .world_mut()
        .resource_mut::<DataRegistry>()
        .load::<ItemConfig>(&server, "datatables/ItemConfig.json");
    assert!(wait_until_loaded(&mut app, 60), "加载应在帧上限内完成");
    assert!(
        app.world()
            .resource::<DataRegistry>()
            .get::<ItemConfig>(&1)
            .is_some(),
        "unload 前应命中"
    );

    app.world_mut()
        .resource_mut::<DataRegistry>()
        .unload::<ItemConfig>();

    let registry = app.world().resource::<DataRegistry>();
    assert!(
        !registry.is_loaded::<ItemConfig>(),
        "unload 后 is_loaded 应为 false"
    );
    assert!(
        registry.get::<ItemConfig>(&1).is_none(),
        "unload 后查询必须 miss（B2-4）"
    );
}

/// 验收「reload 生命周期」：load → unload → reload（复用记录的路径，强制重读磁盘）→
/// 重新入注册表，查询命中。asset 缓存场景下普通 load 不会重发事件——reload 语义由
/// `AssetServer::reload` 强制重读保证（见 lib.rs `DataRegistry::reload` 文档）。
#[test]
fn reload_repopulates_registry_after_unload() {
    let mut app = test_app();
    let server = app.world().resource::<AssetServer>().clone();
    let _handle = app
        .world_mut()
        .resource_mut::<DataRegistry>()
        .load::<ItemConfig>(&server, "datatables/ItemConfig.json");
    assert!(wait_until_loaded(&mut app, 60), "首次加载应在帧上限内完成");
    app.world_mut()
        .resource_mut::<DataRegistry>()
        .unload::<ItemConfig>();
    assert!(
        !app.world()
            .resource::<DataRegistry>()
            .is_loaded::<ItemConfig>(),
        "unload 后应未加载"
    );

    let _handle = app
        .world_mut()
        .resource_mut::<DataRegistry>()
        .reload::<ItemConfig>(&server)
        .expect("已记录路径的 reload 应返回 handle");
    assert!(
        wait_until_loaded(&mut app, 60),
        "reload 后应在帧上限内重新加载"
    );

    let row = app
        .world()
        .resource::<DataRegistry>()
        .get::<ItemConfig>(&1)
        .expect("reload 后应命中");
    assert_eq!(row.name, "fireball");
}

/// reload 未记录过路径 → None（与 `load` 前置的契约）。
#[test]
fn reload_without_prior_load_returns_none() {
    let mut app = test_app();
    let server = app.world().resource::<AssetServer>().clone();
    let handle = app
        .world_mut()
        .resource_mut::<DataRegistry>()
        .reload::<ItemConfig>(&server);
    assert!(handle.is_none(), "未记录路径的 reload 应返回 None");
}
