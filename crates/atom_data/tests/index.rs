//! 索引系统集成测试（纯逻辑，不启动 Bevy App）。
//!
//! spec 依据：`.dsh/plans/atom-data.md` §4.4 测试设计——索引构建 / 多索引 / 复合键 / 多值 /
//! 无索引 / 索引重复键；关键决策 D1（行类型 + `DataTable<T>` 泛型表）、D2（索引形态表）。
//!
//! ## 从 spec 推断的 API 契约（RED 锁定，实现必须遵守）
//!
//! 1. `#[derive(serde::Deserialize, DataAsset)]` 于行类型上生成 `DataIndexed` impl，
//!    `#[index(...)]` 是 `DataAsset` derive 声明的 helper attribute（D2）。
//! 2. 查询方法命名（D2 语义表 + Bevy 生态惯例 `get`/`get_all`/`iter`）：
//!    - 主索引 `#[index(key = "id")]`      → `get(&K) -> Option<&T>`（D2：单键唯一 `get(&K)`）
//!    - 次索引 `#[index(key = "name")]`    → `get_by_name(&K) -> Option<&T>`（K = 字段类型 String）
//!    - 复合键 `#[index(key = ("a","b"))]` → `get_by_pair(&(A, B)) -> Option<&T>`
//!      （2 元组键；若未来支持 n 元组键需重新评估命名——B1-4 实现时注意）
//!    - 多值   `#[index(key = "type", multi)]` → `get_all_by_type(&K) -> Vec<&T>`（D2：`get_all(&K)`）
//!    - 无索引 → `iter()` 全量迭代（D2）
//! 3. 行字段 `r#type`（raw identifier）：宏生成的查询方法名必须剥离 `r#` 前缀 →
//!    `get_all_by_type`（否则生成 `get_all_by_r#type` 无法编译——宏盲区探测）。
//! 4. `DataTable::from_rows(Vec<T>) -> Result<Self, String>`：不依赖 Asset/Bevy 的纯逻辑构造路径
//!    （测试约束：优先纯逻辑）。返回 `Result`：**唯一索引（非 multi）重复 key = 数据错误 → error 分支**
//!    （§4.4 重复键行为锁定；理由：静默 last-wins 会掩盖数据错误，multi 索引天然允许多行共享 key 不报错）。
//! 5. `DataTable<T>: Debug`（derive 即可，Bevy Asset 容器惯例）——`Result::expect_err` 断言
//!    要求 Ok 类型实现 `Debug`，重复键测试据此锁定。
//! 6. `iter()` 保持行插入顺序（`rows: Vec<T>` 语义，D1）。

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

/// 无索引行类型（D2：不加 `#[index]` → 仅 `iter()` 全量迭代）。
#[derive(serde::Deserialize, DataAsset)]
struct PlainRow {
    id: i32,
    name: String,
}

fn sample_rows() -> Vec<AbilityConfig> {
    vec![
        AbilityConfig {
            id: 1,
            name: "fireball".to_string(),
            a: 1,
            b: 2,
            r#type: 1,
        },
        AbilityConfig {
            id: 2,
            name: "ice".to_string(),
            a: 3,
            b: 4,
            r#type: 2,
        },
        AbilityConfig {
            id: 3,
            name: "thunder".to_string(),
            a: 5,
            b: 6,
            r#type: 1,
        },
    ]
}

/// §4.4「索引构建」：`#[index(key = "id")]` → `get(&1)` 命中 / 未命中 None。
#[test]
fn primary_get_hit_and_miss() {
    let table = DataTable::from_rows(sample_rows()).expect("合法数据构建索引不应失败");

    let hit = table.get(&1).expect("主索引 id=1 应命中");
    assert_eq!(hit.id, 1);
    assert_eq!(hit.name, "fireball");
    assert_eq!(hit.r#type, 1, "多值索引字段经 raw identifier 访问应正确");

    assert!(table.get(&999).is_none(), "不存在的 id 应返回 None");
}

/// §4.4「多索引」：`#[index(key = "name")]` 独立次索引按 name 查询。
#[test]
fn secondary_name_index_get() {
    let table = DataTable::from_rows(sample_rows()).expect("合法数据构建索引不应失败");

    let hit = table
        .get_by_name(&"ice".to_string())
        .expect("次索引 name=ice 应命中");
    assert_eq!(hit.id, 2);

    assert!(
        table.get_by_name(&"nope".to_string()).is_none(),
        "未知 name 应返回 None"
    );
}

/// §4.4「复合键」：`#[index(key = ("a", "b"))]` → 元组 key `(A, B)` 查询。
#[test]
fn composite_key_get_by_pair() {
    let table = DataTable::from_rows(sample_rows()).expect("合法数据构建索引不应失败");

    let hit = table
        .get_by_pair(&(3, 4))
        .expect("复合键 (a,b)=(3,4) 应命中");
    assert_eq!(hit.id, 2);

    assert!(
        table.get_by_pair(&(3, 99)).is_none(),
        "复合键未命中应返回 None"
    );
}

/// §4.4「多值」：`multi` 一 key 多行，`get_all` 返回全部匹配行且保持行插入顺序。
#[test]
fn multi_index_get_all_returns_all_matches_in_row_order() {
    let table = DataTable::from_rows(sample_rows()).expect("合法数据构建索引不应失败");

    let matches = table.get_all_by_type(&1);
    assert_eq!(matches.len(), 2, "type=1 应命中 2 行");
    assert_eq!(matches[0].id, 1, "应保持行插入顺序");
    assert_eq!(matches[1].id, 3, "应保持行插入顺序");
}

/// 盲区探测：多值索引未知 key 返回**空 Vec**（而非 None——`get_all` 语义是集合）。
#[test]
fn multi_index_get_all_unknown_key_returns_empty() {
    let table = DataTable::from_rows(sample_rows()).expect("合法数据构建索引不应失败");

    assert!(
        table.get_all_by_type(&7).is_empty(),
        "未知 type 应返回空 Vec"
    );
}

/// §4.4「无索引」：不加 `#[index]` → `iter()` 全量迭代（保持插入顺序）。
#[test]
fn no_index_table_iterates_all_rows() {
    let rows = vec![
        PlainRow {
            id: 1,
            name: "a".to_string(),
        },
        PlainRow {
            id: 2,
            name: "b".to_string(),
        },
    ];
    let table = DataTable::from_rows(rows).expect("无索引表构建不应失败");

    let ids: Vec<i32> = table.iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 2], "iter() 应按插入顺序全量迭代");
}

/// §4.4「索引重复键」：唯一索引重复 key → **error**（锁定 error 分支，拒绝静默 last-wins）。
#[test]
fn duplicate_unique_key_is_rejected() {
    let rows = vec![
        AbilityConfig {
            id: 1,
            name: "fireball".to_string(),
            a: 1,
            b: 2,
            r#type: 1,
        },
        AbilityConfig {
            id: 1, // 与上一行重复的唯一主索引 key
            name: "duplicate".to_string(),
            a: 9,
            b: 9,
            r#type: 9,
        },
    ];

    let err = DataTable::from_rows(rows).expect_err("唯一索引重复 key 必须被拒绝（error 分支）");
    assert!(
        !err.is_empty(),
        "错误信息不应为空（建议包含重复 key 的值与索引字段名）"
    );
}

/// 盲区探测：multi 索引允许一 key 多行——重复 key 在 multi 索引下**不得**报错。
#[test]
fn duplicate_multi_key_is_allowed() {
    let rows = vec![
        AbilityConfig {
            id: 1,
            name: "fireball".to_string(),
            a: 1,
            b: 2,
            r#type: 1,
        },
        AbilityConfig {
            id: 3,
            name: "thunder".to_string(),
            a: 5,
            b: 6,
            r#type: 1, // 与上一行共享 multi key
        },
    ];

    let table = DataTable::from_rows(rows).expect("multi 索引允许一 key 多行，不应报错");
    assert_eq!(table.get_all_by_type(&1).len(), 2);
}

/// 盲区探测（空输入）：空表所有查询形态均返回空/None，构建不 panic。
#[test]
fn empty_table_returns_none_for_every_query() {
    let table: DataTable<AbilityConfig> = DataTable::from_rows(vec![]).expect("空表构建不应失败");

    assert!(table.get(&1).is_none());
    assert!(table.get_by_name(&"x".to_string()).is_none());
    assert!(table.get_by_pair(&(1, 2)).is_none());
    assert!(table.get_all_by_type(&1).is_empty());
    assert_eq!(table.iter().count(), 0);
}

/// §4.4「多索引」：同一张表上所有索引形态（主/次/复合/多值/全量）独立共存、互不干扰。
#[test]
fn all_index_forms_coexist_on_one_table() {
    let table = DataTable::from_rows(sample_rows()).expect("合法数据构建索引不应失败");

    assert_eq!(table.get(&2).expect("id=2 命中").name, "ice");
    assert_eq!(
        table
            .get_by_name(&"thunder".to_string())
            .expect("name 命中")
            .id,
        3
    );
    assert_eq!(
        table.get_by_pair(&(1, 2)).expect("复合键命中").name,
        "fireball"
    );
    assert_eq!(table.get_all_by_type(&2).len(), 1);
    assert_eq!(table.iter().count(), 3, "全量迭代应覆盖所有行");
}
