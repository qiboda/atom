//! 原子表类型数据层测试（纯逻辑，不启动 Bevy App）。
//!
//! spec 依据：`.dsh/plans/atom-data.md` §6（issue #5 Batch 3）——
//! - §6.4「AbilityConfig/BuffConfig/LayerTagConfig 反序列化 + 索引查询」
//! - §6.4「layertag 跨表引用解析（Ability 引用 LayerTagConfig）」
//! - B3-1（serde 结构体 + `#[derive(DataAsset)]` 定义原子表类型）
//! - B3-4（data_ref 声明化跨表引用）
//! - §6.3 验收「数据访问全部经 DataRegistry + DataAsset 声明」
//! - 行为不变：字段与现有 Luban Bean（`atom_datatables/gen/atom_cfg/src/effect.rs` 的
//!   Ability/Buff、`layertag.rs` 的 LayerTag）一致
//!
//! ## 从 spec 推断的 API 契约（RED 锁定，GREEN 必须遵守）
//!
//! 1. 三个表类型定义于 `atom_ability::config`（生产模块，B3-1）：
//!    - `AbilityConfig`：主索引 `#[index(key = "id")]`，字段与 Luban Ability 一致
//!      （id/name/desc/graph_class/activation_type/cd + 6 个 layertag 字段）
//!    - `BuffConfig`：主索引 `#[index(key = "id")]`，字段与 Luban Buff 一致
//!      （含 max_layer/duration/interval + 6 个 layertag 字段）
//!    - `LayerTagConfig`：主索引 `#[index(key = "raw_layertag")]`（**String 主键**——
//!      plans D3 锁定「现有跨表引用全是 String 键」：`start_required_layertags` 列表
//!      元素即 raw layertag 字符串，直接按字符串键解析，行为等价于现状
//!      `StateLayerTagRegistry::request_from_raw`；prompt 示例 `key = "id"` 与此矛盾，
//!      以数据语义为准——见 QA 报告的「关键问题」）
//!    - `AbilityType`：serde 枚举（`Active`/`Passive`，JSON 字符串形态）
//!    - `RevertableLayerTag`：在 atom_ability 内重定义（`raw_layertag: String` +
//!      `revertable: bool`），不再从 atom_datatables 导入
//! 2. 4 个 `Vec<String>` layertag 字段（start_required/start_disabled/abort_required/
//!    abort_disabled）声明 `#[data_ref(table = "LayerTagConfig", key = "raw_layertag")]`，
//!    宏生成 `resolve_{field}(&self, &DataRegistry) -> Option<Vec<&LayerTagConfig>>`
//!    惰性解析（D3：目标表未加载 → None；已加载 → 逐键解析、键缺失跳过、保持顺序）。
//!    `start_added/removed_layertags: Vec<RevertableLayerTag>` **不**声明 data_ref——
//!    元素是内嵌结构而非键列表（宏只支持 String 键列表形态）。
//! 3. 跨 crate 查询限制：宏生成的 `{Row}Queries` trait 是**私有**（宏生成 `trait` 无
//!    `pub`），集成测试（tests/）无法把 trait 引入 scope → 查询必须走公共 API：
//!    `DataTable::get_primary(&pk)`（inherent）与 `DataRegistry::get::<T>(&pk)`。

use atom_ability::config::{
    AbilityConfig, AbilityType, BuffConfig, LayerTagConfig, RevertableLayerTag,
};
use atom_data::{DataRegistry, DataTable};

/// AbilityConfig 的 JSON 数组（与 Luban Bean 字段一致；activation_type 为 serde 字符串枚举）。
const ABILITY_JSON: &str = r#"[
  {
    "id": 101,
    "name": "fireball",
    "desc": "投掷火球",
    "graph_class": "base_attack",
    "activation_type": "Active",
    "cd": 3.5,
    "start_required_layertags": ["fire", "burn"],
    "start_disabled_layertags": ["stun"],
    "start_added_layertags": [{ "raw_layertag": "burning", "revertable": true }],
    "start_removed_layertags": [{ "raw_layertag": "wet", "revertable": false }],
    "abort_required_layertags": [],
    "abort_disabled_layertags": ["silence"]
  },
  {
    "id": 102,
    "name": "ice",
    "desc": "冰霜",
    "graph_class": "base_attack",
    "activation_type": "Passive",
    "cd": 5.0,
    "start_required_layertags": [],
    "start_disabled_layertags": ["burn"],
    "start_added_layertags": [],
    "start_removed_layertags": [],
    "abort_required_layertags": ["wet"],
    "abort_disabled_layertags": []
  }
]"#;

/// BuffConfig 的 JSON 数组。
const BUFF_JSON: &str = r#"[
  {
    "id": 201,
    "name": "poison",
    "desc": "持续中毒",
    "graph_class": "buff_poison",
    "max_layer": 3,
    "duration": 10.0,
    "interval": 2.0,
    "start_required_layertags": [],
    "start_disabled_layertags": [],
    "start_added_layertags": [{ "raw_layertag": "poisoned", "revertable": true }],
    "start_removed_layertags": [],
    "abort_required_layertags": [],
    "abort_disabled_layertags": []
  }
]"#;

/// LayerTagConfig 的 JSON 数组（主键 = raw_layertag 字符串）。
const LAYER_TAG_JSON: &str = r#"[
  { "raw_layertag": "fire", "desc": "火焰", "counter": true },
  { "raw_layertag": "burn", "desc": "灼烧", "counter": true },
  { "raw_layertag": "stun", "desc": "眩晕", "counter": false },
  { "raw_layertag": "wet", "desc": "潮湿", "counter": true },
  { "raw_layertag": "silence", "desc": "沉默", "counter": false },
  { "raw_layertag": "burning", "desc": "燃烧中", "counter": true }
]"#;

/// 结构字面量构造（锁定字段名 + 字段类型，与 JSON 反序列化互为补充验证）。
fn ability_fireball() -> AbilityConfig {
    AbilityConfig {
        id: 101,
        name: "fireball".to_string(),
        desc: "投掷火球".to_string(),
        graph_class: "base_attack".to_string(),
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
        abort_required_layertags: vec![],
        abort_disabled_layertags: vec!["silence".to_string()],
    }
}

fn layer_tag_table() -> DataTable<LayerTagConfig> {
    serde_json::from_str(LAYER_TAG_JSON).expect("LayerTagConfig JSON 应反序列化")
}

/// §6.4「反序列化 + 索引查询」：AbilityConfig JSON → `DataTable`，主索引命中且全字段正确。
#[test]
fn ability_config_json_deserializes_all_fields() {
    let table: DataTable<AbilityConfig> =
        serde_json::from_str(ABILITY_JSON).expect("AbilityConfig JSON 应反序列化为 DataTable");

    assert_eq!(table.len(), 2);
    let row = table
        .get_primary(&101)
        .expect("主索引 id=101 应命中（索引已构建）");
    assert_eq!(row.name, "fireball");
    assert_eq!(row.desc, "投掷火球");
    assert_eq!(row.graph_class, "base_attack");
    assert_eq!(row.activation_type, AbilityType::Active);
    assert_eq!(row.cd, 3.5);
    assert_eq!(row.start_required_layertags, vec!["fire", "burn"]);
    assert_eq!(row.start_disabled_layertags, vec!["stun"]);
    assert_eq!(
        row.start_added_layertags,
        vec![RevertableLayerTag {
            raw_layertag: "burning".to_string(),
            revertable: true,
        }]
    );
    assert_eq!(
        row.start_removed_layertags,
        vec![RevertableLayerTag {
            raw_layertag: "wet".to_string(),
            revertable: false,
        }]
    );
    assert_eq!(row.abort_required_layertags, Vec::<String>::new());
    assert_eq!(row.abort_disabled_layertags, vec!["silence"]);

    let passive = table
        .get_primary(&102)
        .expect("主索引 id=102 应命中");
    assert_eq!(passive.activation_type, AbilityType::Passive);
    assert_eq!(passive.start_required_layertags, Vec::<String>::new());
    assert_eq!(passive.abort_required_layertags, vec!["wet"]);

    assert!(
        table.get_primary(&999).is_none(),
        "不存在的 id 应返回 None"
    );
}

/// §6.4「反序列化 + 索引查询」：BuffConfig JSON → `DataTable`，主索引命中且数值字段正确。
#[test]
fn buff_config_json_deserializes_all_fields() {
    let table: DataTable<BuffConfig> =
        serde_json::from_str(BUFF_JSON).expect("BuffConfig JSON 应反序列化为 DataTable");

    let row = table.get_primary(&201).expect("主索引 id=201 应命中");
    assert_eq!(row.name, "poison");
    assert_eq!(row.desc, "持续中毒");
    assert_eq!(row.graph_class, "buff_poison");
    assert_eq!(row.max_layer, 3);
    assert_eq!(row.duration, 10.0);
    assert_eq!(row.interval, 2.0);
    assert_eq!(
        row.start_added_layertags,
        vec![RevertableLayerTag {
            raw_layertag: "poisoned".to_string(),
            revertable: true,
        }]
    );
}

/// §6.4「反序列化 + 索引查询」：LayerTagConfig 主键为 raw_layertag 字符串。
#[test]
fn layer_tag_config_json_deserializes_and_queries_by_raw_layertag() {
    let table = layer_tag_table();

    let row = table
        .get_primary(&"fire".to_string())
        .expect("raw_layertag 字符串主索引应命中");
    assert_eq!(row.desc, "火焰");
    assert_eq!(row.counter, true);

    let row = table
        .get_primary(&"silence".to_string())
        .expect("raw_layertag 字符串主索引应命中");
    assert_eq!(row.counter, false);

    assert!(
        table.get_primary(&"ghost".to_string()).is_none(),
        "未在数据表中的 raw layertag 应返回 None"
    );
}

/// B3-1 数据完整性：唯一主索引重复 id 必须被拒绝（沿用 atom_data D2 锁定行为）。
#[test]
fn duplicate_ability_id_is_rejected() {
    let dup = r#"[
      { "id": 101, "name": "a", "desc": "", "graph_class": "g", "activation_type": "Active", "cd": 1.0,
        "start_required_layertags": [], "start_disabled_layertags": [], "start_added_layertags": [],
        "start_removed_layertags": [], "abort_required_layertags": [], "abort_disabled_layertags": [] },
      { "id": 101, "name": "b", "desc": "", "graph_class": "g", "activation_type": "Active", "cd": 1.0,
        "start_required_layertags": [], "start_disabled_layertags": [], "start_added_layertags": [],
        "start_removed_layertags": [], "abort_required_layertags": [], "abort_disabled_layertags": [] }
    ]"#;
    let result: Result<DataTable<AbilityConfig>, _> = serde_json::from_str(dup);
    assert!(result.is_err(), "重复唯一主索引必须被拒绝（数据完整性）");
}

/// 构造含两表的 registry（引用方 AbilityConfig + 被引用方 LayerTagConfig）。
fn registry_with_both_tables() -> DataRegistry {
    let mut registry = DataRegistry::default();
    registry.insert(
        DataTable::from_rows(vec![ability_fireball()]).expect("合法数据构建索引不应失败"),
    );
    registry.insert(layer_tag_table());
    registry
}

/// B3-4 + D3「跨表引用解析」：两表均已加载 → `resolve_start_required_layertags` 逐键解析、
/// 保持引用列表顺序（["fire", "burn"] → 两行）。
#[test]
fn resolve_start_required_layertags_returns_rows_in_field_order() {
    let registry = registry_with_both_tables();
    let ability = registry
        .get::<AbilityConfig>(&101)
        .expect("引用方表已加载应命中");

    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some");

    assert_eq!(resolved.len(), 2, "引用列表 2 个键应解析出 2 行");
    let raw_tags: Vec<&str> = resolved.iter().map(|t| t.raw_layertag.as_str()).collect();
    assert_eq!(raw_tags, vec!["fire", "burn"], "应按引用列表顺序解析");
    assert_eq!(resolved[0].desc, "火焰", "解析结果应携带目标表行数据");
}

/// B3-4：abort_required 与 abort_disabled 的 resolve 方法同样存在且解析正确
/// （4 个 `Vec<String>` 字段全部声明 data_ref）。
#[test]
fn resolve_abort_and_disabled_layertags_fields() {
    let registry = registry_with_both_tables();
    let ability = registry
        .get::<AbilityConfig>(&101)
        .expect("引用方表已加载应命中");

    let disabled = ability
        .resolve_start_disabled_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some");
    let raw_tags: Vec<&str> = disabled.iter().map(|t| t.raw_layertag.as_str()).collect();
    assert_eq!(raw_tags, vec!["stun"]);

    let abort_required = ability
        .resolve_abort_required_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some");
    assert!(
        abort_required.is_empty(),
        "空引用列表 → Some(空)（区别于未加载的 None）"
    );

    let abort_disabled = ability
        .resolve_abort_disabled_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some");
    let raw_tags: Vec<&str> = abort_disabled
        .iter()
        .map(|t| t.raw_layertag.as_str())
        .collect();
    assert_eq!(raw_tags, vec!["silence"]);
}

/// Q8 惰性：仅引用方已加载、目标表未加载 → 解析返回 None。
#[test]
fn resolve_returns_none_when_target_not_loaded() {
    let mut registry = DataRegistry::default();
    registry.insert(
        DataTable::from_rows(vec![ability_fireball()]).expect("合法数据构建索引不应失败"),
    );

    let ability = registry
        .get::<AbilityConfig>(&101)
        .expect("引用方表已加载应命中");
    assert!(
        ability.resolve_start_required_layertags(&registry).is_none(),
        "目标表未加载时解析必须返回 None（惰性，Q8）"
    );
}

/// §6.4 盲区：引用列表含目标表不存在的键 → 跳过（返回存在子集，不 panic）。
#[test]
fn resolve_skips_keys_missing_in_target() {
    let registry = registry_with_both_tables();

    let ability = AbilityConfig {
        start_required_layertags: vec![
            "fire".to_string(),
            "ghost".to_string(),
            "burn".to_string(),
        ],
        ..ability_fireball()
    };

    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some（即使部分键缺失）");
    let raw_tags: Vec<&str> = resolved.iter().map(|t| t.raw_layertag.as_str()).collect();
    assert_eq!(raw_tags, vec!["fire", "burn"], "缺失键应被跳过");
}

/// B3-4 盲区（load 顺序无关，§5.4 语义延续）：被引用方先加载、引用方后加载 → 解析仍成功。
#[test]
fn resolve_succeeds_regardless_of_load_order() {
    let mut registry = DataRegistry::default();
    registry.insert(layer_tag_table());
    registry.insert(
        DataTable::from_rows(vec![ability_fireball()]).expect("合法数据构建索引不应失败"),
    );

    let ability = registry
        .get::<AbilityConfig>(&101)
        .expect("引用方表已加载应命中");
    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("两表均已加载，解析应成功");
    assert_eq!(resolved.len(), 2);
}
