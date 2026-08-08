//! 全格式加载示例（B1-5 + B1-6）：同一 `DataTable<T>` 注册 json/ron/toml 三种 loader，
//! 展示「格式由使用方选择」（Q3）与目录约定 `datatables/<表类型名>.<ext>`（Q10 / B1-6）。
//!
//! 目录约定：`assets/datatables/<表类型名>.json`——文件名与行类型名一致
//! （如 `ItemConfig` → `datatables/ItemConfig.json`），扩展名决定格式。
//!
//! 运行：`cargo run -p atom_data --example full_formats --release`
//!
//! 流程：Startup 用 `AssetServer` 加载三种格式的同构数据 → Update 监听
//! `AssetEvent::LoadedWithDependencies`（本体 + 全部依赖就绪信号）→ 从
//! `Assets<DataTable<T>>` 取出表，展示主索引 `get(&1)` 与多值索引 `get_all_by_kind(&1)` 查询。

use atom_data::{DataAsset, DataTable};
use bevy::app::AppExit;
use bevy::asset::{AssetEvent, AssetServer, Assets};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_common_assets::toml::TomlAssetPlugin;

/// 示例行类型：主索引 `id` + 多值索引 `kind`（目录约定：文件名 = 表类型名 `ItemConfig`）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
#[index(key = "kind", multi)]
struct ItemConfig {
    id: i32,
    name: String,
    kind: i32,
}

/// 三种格式的加载句柄（B1-5：格式由使用方插件注册选择，框架不绑定）。
#[derive(Resource)]
struct DemoHandles {
    json: Handle<DataTable<ItemConfig>>,
    ron: Handle<DataTable<ItemConfig>>,
    toml: Handle<DataTable<ItemConfig>>,
}

/// 已加载并验证的格式计数（全部 3 种格式 LoadedWithDependencies 后退出示例）。
#[derive(Resource, Default)]
struct LoadedCount(u32);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(AssetPlugin::default())
        // B1-5：同类型注册多个格式插件（扩展名路由到对应 loader）；init_asset 幂等可省。
        .init_asset::<DataTable<ItemConfig>>()
        .add_plugins(JsonAssetPlugin::<DataTable<ItemConfig>>::new(&["json"]))
        .add_plugins(RonAssetPlugin::<DataTable<ItemConfig>>::new(&["ron"]))
        .add_plugins(TomlAssetPlugin::<DataTable<ItemConfig>>::new(&["toml"]))
        .add_systems(Startup, load_all_formats)
        .init_resource::<LoadedCount>()
        .add_systems(Update, query_loaded_tables)
        .run();
}

/// Startup：按目录约定 `datatables/<表类型名>.<ext>` 加载三种格式。
fn load_all_formats(asset_server: Res<AssetServer>, mut commands: Commands) {
    let json = asset_server.load::<DataTable<ItemConfig>>("datatables/ItemConfig.json");
    let ron = asset_server.load::<DataTable<ItemConfig>>("datatables/ItemConfig.ron");
    let toml = asset_server.load::<DataTable<ItemConfig>>("datatables/ItemConfig.toml");
    info!("full_formats: 已请求加载 3 个格式（datatables/ItemConfig.{{json,ron,toml}}）");
    commands.insert_resource(DemoHandles { json, ron, toml });
}

/// Update：`LoadedWithDependencies` = 依赖就绪信号（Q8 惰性查询挂载点），
/// 展示主索引 `get(&id)` 与多值索引 `get_all_by_kind(&kind)`。
fn query_loaded_tables(
    mut events: MessageReader<AssetEvent<DataTable<ItemConfig>>>,
    assets: Res<Assets<DataTable<ItemConfig>>>,
    handles: Option<Res<DemoHandles>>,
    mut loaded: ResMut<LoadedCount>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(handles) = handles else { return };
    for event in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };
        let format = if *id == handles.json.id() {
            "json"
        } else if *id == handles.ron.id() {
            "ron"
        } else if *id == handles.toml.id() {
            "toml"
        } else {
            continue;
        };
        let table = assets
            .get(*id)
            .expect("LoadedWithDependencies 后资产应已可查询");
        let by_id = table.get(&1).expect("主索引 id=1 应命中");
        let by_kind = table.get_all_by_kind(&1);
        info!(
            "full_formats[{format}]: 加载成功，{} 行；get(&1)={:?}；get_all_by_kind(&1)={:?}",
            table.len(),
            by_id.name,
            by_kind.iter().map(|r| r.id).collect::<Vec<_>>(),
        );
        loaded.0 += 1;
    }
    if loaded.0 >= 3 {
        info!("full_formats: 3 种格式全部加载并查询成功，示例正常退出");
        exit.write(AppExit::Success);
    }
}
