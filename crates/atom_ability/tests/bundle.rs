//! Bundle 重构（B3-3）与行为回归测试。
//!
//! spec 依据：`.omo/plans/atom-data.md` §6（issue #5 Batch 3）——
//! - §6.4「Bundle 构造：能力/增益 spawn 后数据正确（graph_class、layertag 解析）」
//! - B3-3「`TbAbilityRow`/`TbBuffRow`（key+data 分离组件）删除 → 数据直接 + 索引；
//!   `AbilityBundle`/`BuffBundle` 改为携带行数据/键」（handoff 锁定）
//! - 行为不变：`AbilityBundle::new` 后 layertag 解析正确（经 StateLayerTagRegistry）；
//!   `trigger_ability_add`/`trigger_buff_on_add` observer 读 graph_class →
//!   EffectGraphAddEvent（数据源从 TbAbilityRow.data 改为新数据形态）；
//!   `init_state_layertag_registry` 用 LayerTagConfig 数据填充 registry
//!
//! ## 从 spec 推断的 API 契约（RED 锁定，GREEN 必须遵守）
//!
//! 1. `AbilityBundle::new(config: &AbilityConfig, state_registry: &StateLayerTagRegistry)
//!    -> Self`；`BuffBundle::new(config: &BuffConfig, state_registry: &StateLayerTagRegistry)
//!    -> Self`——**普通引用**而非 `&Res<...>`（现签名）：`Res` 字段是 `pub(crate)` 私有、
//!    测试无法在 App 外构造（已查 Bevy 0.19 源码），且 Bundle 构造无需 system-param 语义；
//!    迁移本就是签名重构时机（TbAbilityRow → config）。生产调用方在系统内经 `&*res` 适配。
//! 2. 行为等价（§「行为不变」）：layertag 解析语义与现 `AbilityStartTagBundle::new` /
//!    `BuffStartTagBundle::new` 一致——已注册标签进入容器、未注册跳过（warn 不 panic）；
//!    buff 的 `buff_time`（duration/interval，interval=0 → 无 looper）与现
//!    `BuffBundle::new` 一致。
//! 3. observer 回归：spawn 迁移后 Bundle → `On<Add, Ability>`/`On<Add, Buff>` 读取
//!    新数据形态 → 发出 `EffectGraphAddEvent { graph_class, ability_entity }`，
//!    graph_class 等于 config 的 graph_class 字段（数据源迁移锁定）。
//! 4. `init_state_layertag_registry`：DataRegistry 注入 LayerTagConfig 表后一帧内
//!    StateLayerTagRegistry 填充全部行 raw_layertag。

use atom_ability::{
    ability::bundle::spawn_ability,
    buff::bundle::spawn_buff,
    config::{AbilityConfig, AbilityType, BuffConfig, RevertableLayerTag},
    graph::event::EffectGraphAddEvent,
    stateset::StateLayerTagRegistry,
    AbilitySubsystemPlugin,
};
use atom_data::{DataRegistry, DataRegistryPlugin, DataTable};
use bevy::{asset::AssetPlugin, prelude::*, scene::ScenePlugin};

fn ability_config() -> AbilityConfig {
    AbilityConfig {
        id: 101,
        name: "fireball".to_string(),
        desc: "投掷火球".to_string(),
        graph_class: "test_graph".to_string(),
        activation_type: AbilityType::Active,
        cd: 3.5,
        start_required_layertags: vec!["fire".to_string(), "burn".to_string()],
        start_disabled_layertags: vec!["stun".to_string()],
        start_added_layertags: vec![RevertableLayerTag {
            raw_layertag: "burning".to_string(),
            revertable: true,
        }],
        start_removed_layertags: vec![RevertableLayerTag {
            raw_layertag: "wet".to_string(),
            revertable: false,
        }],
        abort_required_layertags: vec!["wet".to_string()],
        abort_disabled_layertags: vec!["silence".to_string()],
    }
}

fn buff_config() -> BuffConfig {
    BuffConfig {
        id: 201,
        name: "poison".to_string(),
        desc: "持续中毒".to_string(),
        graph_class: "test_buff_graph".to_string(),
        max_layer: 3,
        duration: 10.0,
        interval: 2.0,
        start_required_layertags: vec!["fire".to_string()],
        start_disabled_layertags: vec![],
        start_added_layertags: vec![],
        start_removed_layertags: vec![RevertableLayerTag {
            raw_layertag: "wet".to_string(),
            revertable: true,
        }],
        abort_required_layertags: vec![],
        abort_disabled_layertags: vec![],
    }
}

/// 构造已注册全部引用 raw layertag 的状态层标签注册表。
/// 捕获 EffectGraphAddEvent 的 graph_class 序列（全局 observer，EntityEvent 任意目标均触发）。
#[derive(Resource, Default)]
struct CapturedGraphAdd {
    events: Vec<String>,
}

fn capture_effect_graph_add(
    trigger: On<EffectGraphAddEvent>,
    mut captured: ResMut<CapturedGraphAdd>,
) {
    captured.events.push(trigger.event().graph_class.clone());
}

/// 测试用配置资源（observer 数据源注入 + spawn 系统读取）。
#[derive(Resource)]
struct SpawnConfigs {
    ability: AbilityConfig,
    buff: BuffConfig,
}

fn spawn_ability_bundle(
    mut commands: Commands,
    configs: Res<SpawnConfigs>,
    state_registry: Res<StateLayerTagRegistry>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }
    *spawned = true;
    commands.spawn_scene(spawn_ability(&configs.ability, &state_registry));
}

fn spawn_buff_bundle(
    mut commands: Commands,
    configs: Res<SpawnConfigs>,
    state_registry: Res<StateLayerTagRegistry>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }
    *spawned = true;
    commands.spawn_scene(spawn_buff(&configs.buff, &state_registry));
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ScenePlugin,
        DataRegistryPlugin,
    ));
    app.add_plugins(AbilitySubsystemPlugin);
    app.init_resource::<CapturedGraphAdd>();
    app.world_mut().add_observer(capture_effect_graph_add);
    app
}

/// 行为不变：spawn 迁移后 AbilityBundle → `trigger_ability_add` 读取新数据形态 →
/// EffectGraphAddEvent 携带 config.graph_class（数据源 = AbilityConfig）。
#[test]
fn spawn_ability_bundle_emits_graph_add_with_config_class() {
    let mut app = test_app();
    app.insert_resource(SpawnConfigs {
        ability: ability_config(),
        buff: buff_config(),
    });
    app.add_systems(Update, spawn_ability_bundle);
    // 注入能力表（observer 新数据源：registry 查询或 bundle 携带数据，二者皆需 config 存在）
    app.world_mut().resource_mut::<DataRegistry>().insert(
        DataTable::from_rows(vec![ability_config()]).expect("合法数据构建索引不应失败"),
    );

    for _ in 0..10 {
        app.update();
        if !app.world().resource::<CapturedGraphAdd>().events.is_empty() {
            break;
        }
    }

    let captured = app.world().resource::<CapturedGraphAdd>();
    assert!(
        captured.events.iter().any(|class| class == "test_graph"),
        "spawn 后应发出 graph_class = config 数据的 EffectGraphAddEvent，实际: {:?}",
        captured.events
    );
}

/// 行为不变：spawn 迁移后 BuffBundle → `trigger_buff_on_add` 读取新数据形态 →
/// EffectGraphAddEvent 携带 config.graph_class（数据源 = BuffConfig）。
#[test]
fn spawn_buff_bundle_emits_graph_add_with_config_class() {
    let mut app = test_app();
    app.insert_resource(SpawnConfigs {
        ability: ability_config(),
        buff: buff_config(),
    });
    app.add_systems(Update, spawn_buff_bundle);
    app.world_mut().resource_mut::<DataRegistry>().insert(
        DataTable::from_rows(vec![buff_config()]).expect("合法数据构建索引不应失败"),
    );

    for _ in 0..10 {
        app.update();
        if !app.world().resource::<CapturedGraphAdd>().events.is_empty() {
            break;
        }
    }

    let captured = app.world().resource::<CapturedGraphAdd>();
    assert!(
        captured.events.iter().any(|class| class == "test_buff_graph"),
        "spawn 后应发出 graph_class = config 数据的 EffectGraphAddEvent，实际: {:?}",
        captured.events
    );
}

// --- stateset 迁移（B3-2：init_state_layertag_registry 用 LayerTagConfig 填充 registry） ---

/// 行为不变：DataRegistry 注入 LayerTagConfig 表后，`init_state_layertag_registry`
/// 将每行 raw_layertag 注册进 StateLayerTagRegistry。
#[test]
fn init_state_layertag_registry_populates_from_layer_tag_config() {
    let mut app = test_app();
    let table: DataTable<atom_ability::config::LayerTagConfig> =
        serde_json::from_str(
            r#"[
              { "raw_layertag": "fire", "desc": "火焰", "counter": true },
              { "raw_layertag": "burn", "desc": "灼烧", "counter": true }
            ]"#,
        )
        .expect("LayerTagConfig JSON 应反序列化");
    app.world_mut().resource_mut::<DataRegistry>().insert(table);

    app.update();

    let registry = app.world().resource::<StateLayerTagRegistry>();
    assert!(
        registry.0.request_from_raw("fire").is_some(),
        "LayerTagConfig 行 raw_layertag 应注册进状态层标签注册表"
    );
    assert!(registry.0.request_from_raw("burn").is_some());
    assert!(
        registry.0.request_from_raw("ghost").is_none(),
        "不在数据表中的 raw tag 不应注册"
    );
}
