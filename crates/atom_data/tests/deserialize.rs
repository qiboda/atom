//! 反序列化 + 全格式等价性集成测试（纯 serde，不启动 Bevy App）。
//!
//! spec 依据：`.dsh/plans/atom-data.md` §4.4——反序列化（JSON 数组 → `DataTable<T>`，id/name 字段
//! 正确）、全格式等价性（json/ron/toml 同数据 → 相同查询结果）；D1（`DataTable<T>` Deserialize 手动
//! impl：`Vec<T>` → 构建索引，B1-3）。
//!
//! ## 从 spec 推断的 API 契约（RED 锁定）
//!
//! 1. `DataTable<T>: Deserialize`（B1-3 手动 impl），反序列化后索引已构建（可直接查询）。
//! 2. **格式形态契约**（Q3 全格式 + TOML 语法约束，实证探测见 QA 报告）：
//!    - JSON / RON：**顶层数组**（`[{...}]` / `[(...)]`）→ 行序列
//!    - TOML：根必须是 table，无法表达顶层数组（实证：`toml::from_str::<Vec<T>>` 报
//!      "invalid type: map, expected a sequence"）→ **单键 `rows` map**
//!      （`rows = [ { ... }, ... ]`，也兼容 `[[rows]]` 数组表形态）
//!    - 因此 `DataTable<T>` 的 Deserialize 必须同时接受「顶层序列」与「单键 `rows` map」两种形态。
//! 3. 反序列化错误必须传播（不静默吞错）：重复唯一键 / 缺必填字段 → `Err`。
//! 4. `r#type` 字段（raw identifier）经 serde unraw 映射到 JSON key `"type"`（实证：serde 剥 `r#`）。

use atom_data::{DataAsset, DataTable};

/// 全形态行类型：主索引 id + 次索引 name + 复合键 (a,b) + 多值 type（raw identifier）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
#[index(key = "name")]
#[index(key = ("a", "b"))]
#[index(key = "type", multi)]
struct AbilityConfig {
    id: i32,
    name: String,
    a: i32,
    b: i32,
    r#type: i32,
}

/// 全格式等价性行类型（避开 raw identifier，保证 RON/TOML 语法无歧义）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
#[index(key = "name")]
#[index(key = "kind", multi)]
struct ItemConfig {
    id: i32,
    name: String,
    kind: i32,
}

/// AbilityConfig 的 JSON 数组（顶层数组形态，spec：「JSON 数组 → DataTable<T>」）。
const ABILITY_JSON: &str = r#"[
  {"id":1,"name":"fireball","a":1,"b":2,"type":1},
  {"id":2,"name":"ice","a":3,"b":4,"type":2},
  {"id":3,"name":"thunder","a":5,"b":6,"type":1}
]"#;

/// ItemConfig 三种格式的同构数据（同数据不同格式 → 相同查询结果，§4.4 全格式）。
const ITEM_JSON: &str = r#"[
  {"id":1,"name":"fireball","kind":1},
  {"id":2,"name":"ice","kind":2},
  {"id":3,"name":"thunder","kind":1}
]"#;

const ITEM_RON: &str = r#"[
  (id: 1, name: "fireball", kind: 1),
  (id: 2, name: "ice", kind: 2),
  (id: 3, name: "thunder", kind: 1),
]"#;

/// TOML 根必须是 table——行序列挂在 `rows` 键下（契约见文件头第 2 条）。
const ITEM_TOML: &str = r#"rows = [
  { id = 1, name = "fireball", kind = 1 },
  { id = 2, name = "ice", kind = 2 },
  { id = 3, name = "thunder", kind = 1 },
]"#;

/// §4.4「反序列化」：JSON 数组 → `DataTable<T>`，id/name 字段正确，索引可查询。
#[test]
fn json_array_deserializes_to_indexed_table() {
    let table: DataTable<AbilityConfig> =
        serde_json::from_str(ABILITY_JSON).expect("JSON 数组应反序列化为 DataTable");

    let row = table.get(&1).expect("id=1 应命中（索引已构建）");
    assert_eq!(row.id, 1);
    assert_eq!(row.name, "fireball");
    assert_eq!(row.a, 1);
    assert_eq!(row.b, 2);
    assert_eq!(
        row.r#type, 1,
        "r#type 字段应经 serde unraw 从 JSON key \"type\" 反序列化"
    );

    assert_eq!(
        table
            .get_by_name(&"ice".to_string())
            .expect("次索引 name 应命中")
            .id,
        2
    );

    let typed = table.get_all_by_type(&1);
    assert_eq!(typed.len(), 2, "type=1 应命中 2 行");
    assert_eq!(typed[0].id, 1);
    assert_eq!(typed[1].id, 3);
}

/// 盲区探测（空输入）：`[]` → 空表，查询全空，不 panic。
#[test]
fn json_empty_array_yields_empty_table() {
    let table: DataTable<ItemConfig> = serde_json::from_str("[]").expect("空数组应反序列化为空表");

    assert!(table.get(&1).is_none());
    assert_eq!(table.iter().count(), 0);
}

/// 盲区探测：重复唯一键必须经**反序列化路径**同样被拒绝（error 传播，不静默 last-wins）。
#[test]
fn json_duplicate_unique_key_is_rejected() {
    let dup = r#"[{"id":1,"name":"a","kind":1},{"id":1,"name":"b","kind":2}]"#;
    let result: Result<DataTable<ItemConfig>, _> = serde_json::from_str(dup);
    assert!(result.is_err(), "反序列化路径必须拒绝重复唯一键");
}

/// 盲区探测：缺必填字段的行必须反序列化失败（数据完整性，不静默吞错）。
#[test]
fn json_missing_required_field_is_rejected() {
    let bad = r#"[{"name":"a","kind":1}]"#; // 缺 id 字段
    let result: Result<DataTable<ItemConfig>, _> = serde_json::from_str(bad);
    assert!(result.is_err(), "缺必填字段的行必须被拒绝");
}

/// §4.4「全格式」：ron 与 json 同数据 → 相同查询结果。
#[test]
fn ron_and_json_produce_identical_query_results() {
    let from_json: DataTable<ItemConfig> = serde_json::from_str(ITEM_JSON).expect("json 反序列化");
    let from_ron: DataTable<ItemConfig> = ron::from_str(ITEM_RON).expect("ron 反序列化");

    assert_eq!(from_json.get(&1).expect("json id=1").name, "fireball");
    assert_eq!(from_ron.get(&1).expect("ron id=1").name, "fireball");

    let json_kind1: Vec<i32> = from_json.get_all_by_kind(&1).iter().map(|r| r.id).collect();
    let ron_kind1: Vec<i32> = from_ron.get_all_by_kind(&1).iter().map(|r| r.id).collect();
    assert_eq!(json_kind1, ron_kind1, "json 与 ron 多值查询结果应一致");
}

/// §4.4「全格式」：toml（`rows` 单键 map 形态）与 json 同数据 → 相同查询结果。
#[test]
fn toml_and_json_produce_identical_query_results() {
    let from_json: DataTable<ItemConfig> = serde_json::from_str(ITEM_JSON).expect("json 反序列化");
    let from_toml: DataTable<ItemConfig> = toml::from_str(ITEM_TOML).expect("toml 反序列化");

    assert_eq!(from_toml.get(&2).expect("toml id=2").name, "ice");
    assert_eq!(
        from_json
            .get_by_name(&"thunder".to_string())
            .expect("json name 命中")
            .id,
        from_toml
            .get_by_name(&"thunder".to_string())
            .expect("toml name 命中")
            .id
    );
}

/// §4.4「全格式」：json/ron/toml 三格式同数据 → 相同查询结果（等价性主测试）。
#[test]
fn three_formats_agree_on_query_results() {
    let from_json: DataTable<ItemConfig> = serde_json::from_str(ITEM_JSON).expect("json 反序列化");
    let from_ron: DataTable<ItemConfig> = ron::from_str(ITEM_RON).expect("ron 反序列化");
    let from_toml: DataTable<ItemConfig> = toml::from_str(ITEM_TOML).expect("toml 反序列化");

    for table in [&from_json, &from_ron, &from_toml] {
        assert_eq!(table.iter().count(), 3, "三种格式行数应一致");
        assert_eq!(table.get(&1).expect("id=1 命中").name, "fireball");
        assert_eq!(
            table.get_by_name(&"ice".to_string()).expect("name 命中").id,
            2
        );
        let kind1: Vec<i32> = table.get_all_by_kind(&1).iter().map(|r| r.id).collect();
        assert_eq!(kind1, vec![1, 3], "multi 查询结果应一致且保持行序");
    }
}

/// 盲区探测：`serde` 反序列化路径与 `from_rows` 构造路径查询结果一致（两条构建路径等价）。
#[test]
fn deserialize_path_equals_from_rows_path() {
    let rows = vec![
        ItemConfig {
            id: 1,
            name: "fireball".to_string(),
            kind: 1,
        },
        ItemConfig {
            id: 2,
            name: "ice".to_string(),
            kind: 2,
        },
        ItemConfig {
            id: 3,
            name: "thunder".to_string(),
            kind: 1,
        },
    ];
    let from_rows = DataTable::from_rows(rows).expect("from_rows 构建索引");
    let from_json: DataTable<ItemConfig> = serde_json::from_str(ITEM_JSON).expect("json 反序列化");

    assert_eq!(from_rows.iter().count(), from_json.iter().count());
    assert_eq!(
        from_rows.get(&2).expect("id=2 命中").name,
        from_json.get(&2).expect("id=2 命中").name
    );
    assert_eq!(
        from_rows.get_all_by_kind(&1).len(),
        from_json.get_all_by_kind(&1).len()
    );
}
