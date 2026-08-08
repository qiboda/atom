//! 补充测试：`DataTable<T>` 公共 API 表面覆盖率缺口（QA 独立设计，纯 serde 不启动 Bevy App）。
//!
//! ## 覆盖目标（`cargo llvm-cov nextest -p atom_data --release --show-missing-lines` 定位）
//!
//! | 缺口 | lib.rs 行 | 被测行为 |
//! |------|-----------|----------|
//! | 1 | 139-146 | `DataTable::len` / `is_empty`——现有 35 测试从未直接断言这两个 pub API |
//! | 2 | 154-159 | `impl fmt::Debug for DataTable`——从未断言 `format!("{:?}", table)` 输出 |
//! | 3 | 183-185 | `Visitor::expecting`——仅在类型不匹配（seq/map 之外）时被调用 |
//! | 4 | 207 | `visit_map` 未知键 `IgnoredAny` 路径——现有 map 输入只有 `rows` 单键 |
//! | 5 | 210-211 | 盲区：`missing_field("rows")` 分支 / map 路径重复唯一键错误传播 |
//! | 5 | — | 盲区：`DataRefKey::from_ref_str` 解析失败（非数字键）→ skip（data_ref.rs 未覆盖） |
//!
//! 目标：行覆盖率 85.34% → ≥90%；函数覆盖率 79.17% → ≥85%。
//! 不修改现有 35 测试；不修改 `src/lib.rs`（覆盖率的解法是加测试，不是改代码缩小覆盖）。
//! 尾部 `sync_table` 盲区测试需要 Bevy App（消息通道），其余均为纯 serde 逻辑。

use atom_data::{DataAsset, DataRegistry, DataRegistryPlugin, DataTable};
use bevy::asset::{AssetApp, AssetEvent, AssetId};
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// 覆盖率行类型：主索引 id（与现有测试文件独立的本地定义，integration test 各自成 crate）。
#[derive(serde::Deserialize, DataAsset, Clone)]
#[index(key = "id")]
struct CoverageRow {
    id: i32,
    name: String,
}

/// data_ref 盲区探测：被引用表（主索引 id）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
struct RefTarget {
    id: i32,
    raw: String,
}

/// data_ref 盲区探测：引用方（引用列表元素 = 目标表主键的字符串形式）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
struct RefOwner {
    id: i32,
    #[data_ref(table = "RefTarget", key = "id")]
    refs: Vec<String>,
}

/// 3 行 JSON 顶层数组（spec：JSON 数组形态 → `DataTable<T>`）。
const ROWS_JSON: &str = r#"[
  {"id":1,"name":"fireball"},
  {"id":2,"name":"ice"},
  {"id":3,"name":"thunder"}
]"#;

fn sample_table() -> DataTable<CoverageRow> {
    DataTable::from_rows(vec![
        CoverageRow {
            id: 1,
            name: "fireball".to_string(),
        },
        CoverageRow {
            id: 2,
            name: "ice".to_string(),
        },
        CoverageRow {
            id: 3,
            name: "thunder".to_string(),
        },
    ])
    .expect("合法数据构建索引不应失败")
}

/// 缺口 1：非空表 `len` 返回行数、`is_empty` 为 false（from_rows 构建路径）。
#[test]
fn len_reports_row_count_on_non_empty_table() {
    let table = sample_table();

    assert_eq!(table.len(), 3, "3 行表的 len() 应为 3");
    assert!(!table.is_empty(), "非空表 is_empty() 应为 false");
    assert_eq!(
        table.len(),
        table.rows().len(),
        "len() 应与 rows() 切片长度一致"
    );
    assert_eq!(
        table.len(),
        table.iter().count(),
        "len() 应与 iter() 迭代计数一致"
    );
}

/// 缺口 1：空表 `len` 为 0、`is_empty` 为 true（from_rows 空 Vec）。
#[test]
fn empty_table_len_is_zero_and_is_empty() {
    let table: DataTable<CoverageRow> = DataTable::from_rows(vec![]).expect("空表构建不应失败");

    assert_eq!(table.len(), 0, "空表 len() 应为 0");
    assert!(table.is_empty(), "空表 is_empty() 应为 true");
}

/// 缺口 1：Deserialize 路径与 from_rows 路径的 `len` 一致（双构建路径等价，补齐 len 维度）。
#[test]
fn len_agrees_across_deserialize_and_from_rows_paths() {
    let from_rows = sample_table();
    let from_json: DataTable<CoverageRow> =
        serde_json::from_str(ROWS_JSON).expect("JSON 数组应反序列化为 DataTable");

    assert_eq!(from_rows.len(), 3);
    assert_eq!(
        from_json.len(),
        3,
        "Deserialize 路径 len 应与 from_rows 路径一致"
    );
    assert!(!from_json.is_empty());
}

/// 缺口 1（盲区）：空 JSON 数组 `[]` → 空表，`len` 0 / `is_empty` true。
#[test]
fn empty_json_array_yields_len_zero_and_is_empty() {
    let table: DataTable<CoverageRow> = serde_json::from_str("[]").expect("空数组应反序列化");

    assert_eq!(table.len(), 0, "空 JSON 数组反序列化后 len() 应为 0");
    assert!(
        table.is_empty(),
        "空 JSON 数组反序列化后 is_empty() 应为 true"
    );
}

/// 缺口 2：`format!("{:?}", table)` 输出包含结构名 "DataTable" 与 "len" 字段（值为行数）。
#[test]
fn debug_fmt_contains_table_name_and_len_field() {
    let table = sample_table();
    let debug = format!("{table:?}");

    assert!(
        debug.contains("DataTable"),
        "Debug 输出应含结构名 DataTable，实际: {debug}"
    );
    assert!(
        debug.contains("len: 3"),
        "Debug 输出应含 len 字段且值为行数 3，实际: {debug}"
    );
    assert!(
        debug.contains("index"),
        "Debug 输出应含 index 字段（索引容器 Debug），实际: {debug}"
    );
}

/// 缺口 2（盲区）：空表的 Debug 输出不 panic，且 len 字段为 0。
#[test]
fn debug_fmt_on_empty_table_does_not_panic() {
    let table: DataTable<CoverageRow> = DataTable::from_rows(vec![]).expect("空表构建不应失败");
    let debug = format!("{table:?}");

    assert!(
        debug.contains("len: 0"),
        "空表 Debug 输出应含 len: 0，实际: {debug}"
    );
}

/// 缺口 3：类型不匹配输入（JSON 字符串）→ 反序列化 Err，错误消息含 expecting 契约文本。
#[test]
fn string_input_is_rejected_with_expecting_message() {
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str("\"hello\"");

    let err = result.expect_err("字符串既非 seq 也非 map，必须反序列化失败");
    let msg = err.to_string();
    assert!(
        msg.contains("sequence of rows"),
        "错误消息应含 expecting 契约（a sequence of rows, or a map with a `rows` key），实际: {msg}"
    );
}

/// 缺口 3：类型不匹配输入（JSON 数字）→ Err，错误消息同样含 expecting 契约文本。
#[test]
fn number_input_is_rejected_with_expecting_message() {
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str("42");

    let err = result.expect_err("数字既非 seq 也非 map，必须反序列化失败");
    let msg = err.to_string();
    assert!(
        msg.contains("sequence of rows"),
        "错误消息应含 expecting 契约，实际: {msg}"
    );
}

/// 缺口 3（格式交叉）：RON 字符串输入同样走 expecting 错误路径。
#[test]
fn ron_string_input_is_rejected_with_expecting_message() {
    let result: Result<DataTable<CoverageRow>, _> = ron::from_str("\"hello\"");

    let err = result.expect_err("RON 字符串既非 seq 也非 map，必须反序列化失败");
    let msg = err.to_string();
    assert!(
        msg.contains("sequence of rows"),
        "RON 错误消息应含 expecting 契约，实际: {msg}"
    );
}

/// 缺口 4：JSON map 含未知键（数字/字符串/嵌套对象）→ 未知键被 IgnoredAny 忽略，
/// 反序列化成功且数据正确。
#[test]
fn json_map_with_unknown_keys_is_ignored() {
    let input = r#"{
      "rows": [
        {"id":1,"name":"fireball"},
        {"id":2,"name":"ice"}
      ],
      "extra": 123,
      "another": "ignored",
      "nested": {"deep": [1, 2, 3]}
    }"#;
    let table: DataTable<CoverageRow> =
        serde_json::from_str(input).expect("未知键应被忽略，反序列化成功");

    assert_eq!(table.len(), 2, "rows 键的 2 行应被解析");
    assert_eq!(
        table.get(&2).expect("id=2 命中").name,
        "ice",
        "未知键不影响 rows 数据"
    );
    assert!(table.get(&999).is_none());
}

/// 缺口 4：TOML map（根必须是 table）含 `rows` 之外的其他键 → 未知键被忽略。
#[test]
fn toml_map_with_unknown_keys_is_ignored() {
    let input = r#"
      rows = [
        { id = 1, name = "fireball" },
        { id = 2, name = "ice" },
        { id = 3, name = "thunder" },
      ]
      extra = 123
      flag = true
    "#;
    let table: DataTable<CoverageRow> =
        toml::from_str(input).expect("TOML 未知键应被忽略，反序列化成功");

    assert_eq!(table.len(), 3);
    assert_eq!(
        table.get(&3).expect("id=3 命中").name,
        "thunder",
        "TOML 未知键不影响 rows 数据"
    );
}

/// 缺口 4（键序盲区）：未知键出现在 `rows` 键**之前**同样被忽略（visit_map 循环不依赖键序）。
#[test]
fn json_map_unknown_key_before_rows_is_ignored() {
    let input = r#"{
      "meta": {"version": 1},
      "rows": [
        {"id":1,"name":"fireball"}
      ]
    }"#;
    let table: DataTable<CoverageRow> =
        serde_json::from_str(input).expect("rows 之前的未知键也应被忽略");

    assert_eq!(table.len(), 1);
    assert_eq!(table.get(&1).expect("id=1 命中").name, "fireball");
}

/// 缺口 5（盲区，210 行）：map 形态缺 `rows` 键 → 反序列化 Err（missing field）。
#[test]
fn map_without_rows_key_is_rejected() {
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str(r#"{"foo": 1}"#);

    let err = result.expect_err("缺 rows 键的 map 必须反序列化失败");
    let msg = err.to_string();
    assert!(
        msg.contains("rows"),
        "错误消息应指向缺失的 rows 字段，实际: {msg}"
    );
}

/// 缺口 5（盲区，211 行）：map 形态下 rows 含重复唯一键 → 错误传播（不静默 last-wins）。
#[test]
fn map_rows_with_duplicate_unique_key_is_rejected() {
    let input = r#"{
      "rows": [
        {"id":1,"name":"a"},
        {"id":1,"name":"b"}
      ]
    }"#;
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str(input);

    assert!(
        result.is_err(),
        "map 路径同样必须拒绝重复唯一键（error 分支传播，D2）"
    );
}

/// 缺口 5（盲区，DataRefKey::from_ref_str 失败路径）：引用键含非数字（i32 主键解析失败）
/// → 该键被跳过，返回存在子集；不 panic。
#[test]
fn data_ref_key_parse_failure_is_skipped() {
    let mut registry = DataRegistry::default();
    registry.insert(
        DataTable::from_rows(vec![
            RefTarget {
                id: 1,
                raw: "fire".to_string(),
            },
            RefTarget {
                id: 3,
                raw: "thunder".to_string(),
            },
        ])
        .expect("目标表构建不应失败"),
    );

    // "abc" 无法解析为 i32 → from_ref_str 返回 None → 跳过（与「目标表缺键」的 skip 语义一致）
    let owner = RefOwner {
        id: 1,
        refs: vec!["1".to_string(), "abc".to_string(), "3".to_string()],
    };

    let resolved = owner
        .resolve_refs(&registry)
        .expect("目标表已加载，解析应返回 Some（即使含解析失败键）");
    assert_eq!(resolved.len(), 2, "解析失败键 \"abc\" 应被跳过");
    assert_eq!(resolved[0].id, 1);
    assert_eq!(resolved[0].raw, "fire", "解析出的行应携带完整数据");
    assert_eq!(resolved[1].id, 3);
    assert_eq!(resolved[1].raw, "thunder");
}

/// 缺口 5（盲区，visit_map 错误传播路径）：map 在读取键时中途截断（缺闭合 `}`）
/// → `map.next_key()?` 错误传播为反序列化 Err（不静默吞错）。
#[test]
fn truncated_map_is_rejected_with_error_propagation() {
    let input = r#"{"rows":[{"id":1,"name":"a"}]"#; // 缺外层闭合 }
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str(input);

    assert!(
        result.is_err(),
        "截断的 map 必须在读取键阶段报错（next_key 错误传播）"
    );
}

/// 缺口 5（盲区，visit_map 错误传播路径）：`rows` 键的值类型不是行序列
/// → `map.next_value()?` 错误传播为反序列化 Err。
#[test]
fn rows_key_with_wrong_value_type_is_rejected() {
    let input = r#"{"rows": "not-a-list"}"#;
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str(input);

    assert!(
        result.is_err(),
        "rows 值为字符串（非行序列）必须反序列化失败"
    );
}

/// 缺口 5（盲区，visit_map 错误传播路径）：未知键的值是损坏 JSON（缺值）
/// → `IgnoredAny` 读取失败，错误传播为反序列化 Err（未知键不吞语法错误）。
#[test]
fn unknown_key_with_malformed_value_is_rejected() {
    let input = r#"{"rows":[], "extra": }"#; // extra 的值缺失，语法错误
    let result: Result<DataTable<CoverageRow>, _> = serde_json::from_str(input);

    assert!(
        result.is_err(),
        "未知键的值损坏时不能静默忽略——IgnoredAny 错误必须传播"
    );
}

/// 缺口 5（盲区，sync_table 的 `assets.get` 未命中分支）：`LoadedWithDependencies`
/// 事件携带的资产 id 在 `Assets` 中不存在 → sync 系统跳过（不 panic、不入注册表）。
/// 确定性构造：手动写入指向 `AssetId::invalid()` 的假事件（消息双缓冲，第二帧被读取）。
#[test]
fn sync_table_skips_loaded_event_for_unknown_asset_id() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_resource::<DataRegistry>();
    app.init_asset::<DataTable<CoverageRow>>();
    DataRegistryPlugin::register_table::<CoverageRow>(&mut app);
    app.add_systems(
        Update,
        |mut writer: MessageWriter<AssetEvent<DataTable<CoverageRow>>>| {
            writer.write(AssetEvent::LoadedWithDependencies {
                id: AssetId::<DataTable<CoverageRow>>::invalid(),
            });
        },
    );

    app.update(); // 事件写入
    app.update(); // sync_table 读到事件：assets.get(invalid) → None → 跳过

    assert!(
        !app.world()
            .resource::<DataRegistry>()
            .is_loaded::<CoverageRow>(),
        "指向不存在资产的 LoadedWithDependencies 事件不得入注册表"
    );
}
