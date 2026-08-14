//! DataRegistry 生命周期集成测试（纯逻辑，不启动 Bevy App）。
//!
//! spec 依据：`.dsh/plans/atom-data.md` §5（issue #4 Batch 2）——
//! - §5.4「单表 get/load/unload/is_loaded 生命周期」
//! - §5.4「重复 load 幂等性」
//! - §5.3 验收「`data.get::<T>(id)` 惰性：未加载 None，加载后可查」「`load`/`unload` 显式
//!   生命周期控制（Q9）」
//! - 关键决策 D4（DataRegistry：按行类型 `TypeId` 擦除存储）+ Q10（同步查询）
//!
//! ## 从 spec 推断的 API 契约（RED 锁定，实现必须遵守）
//!
//! 1. `DataRegistry: Default`——可直接构造（无需 Bevy App / AssetServer）；测试经
//!    `registry.insert(table)` 注入 `DataTable<T>`（纯逻辑注册路径）。`load` 走 AssetServer
//!    是 GREEN 的加载路径，测试锁定 registry 的查询语义，不启动 Bevy。
//! 2. `DataIndexed` trait 扩展（Batch 2 设计）：`type PrimaryKey: Hash + Eq + Clone` +
//!    `fn primary_key(&self) -> Self::PrimaryKey`（主索引键提取）；无索引行类型 →
//!    `type PrimaryKey = ();`。
//! 3. `DataRegistry::get::<T>(&T::PrimaryKey) -> Option<&T>`——惰性：未加载返回 None，
//!    加载后按主索引查询（§5.3「未加载 None，加载后可查」）。
//! 4. `DataRegistry::is_loaded::<T>() -> bool` / `unload::<T>()`——显式生命周期（Q9）；
//!    unload 后 get 返回 None（B2-4「unload 后失效」）。
//! 5. 每行类型（`TypeId`）至多一张表：重复 insert 同类型表 = **替换**（registry 级幂等，
//!    §5.4「重复 load 幂等性」的纯逻辑对应——load 内部即 insert，重复 load 不应累积表）。

use atom_data::{DataAsset, DataRegistry, DataTable};

/// 生命周期测试行类型：主索引 id（`PrimaryKey = i32`）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
struct AbilityConfig {
    id: i32,
    name: String,
}

/// 无索引行类型（Batch 2 设计：`PrimaryKey = ()`，仅存续/生命周期管理）。
#[derive(serde::Deserialize, DataAsset)]
struct NoIndexRow {
    id: i32,
    name: String,
}

fn ability_rows() -> Vec<AbilityConfig> {
    vec![
        AbilityConfig {
            id: 1,
            name: "fireball".to_string(),
        },
        AbilityConfig {
            id: 2,
            name: "ice".to_string(),
        },
        AbilityConfig {
            id: 3,
            name: "thunder".to_string(),
        },
    ]
}

fn ability_table() -> DataTable<AbilityConfig> {
    DataTable::from_rows(ability_rows()).expect("合法数据构建索引不应失败")
}

/// §5.4「单表生命周期」+ §5.3「未加载 None」：全新 registry 未加载任何表 →
/// is_loaded false、get 返回 None（惰性语义 Q8）。
#[test]
fn fresh_registry_is_unloaded_and_get_returns_none() {
    let registry = DataRegistry::default();

    assert!(
        !registry.is_loaded::<AbilityConfig>(),
        "未加载前 is_loaded 应为 false"
    );
    assert!(
        registry.get::<AbilityConfig>(&1).is_none(),
        "未加载的表查询必须返回 None（惰性，Q8）"
    );
}

/// §5.4「单表生命周期」+ §5.3「加载后可查」：insert 注入表后 is_loaded true、
/// 主索引命中 / 未命中 None；`primary_key()` 提取主索引字段值。
#[test]
fn insert_then_get_hits_existing_key_and_misses_unknown() {
    let mut registry = DataRegistry::default();
    registry.insert(ability_table());

    assert!(
        registry.is_loaded::<AbilityConfig>(),
        "insert 后 is_loaded 应为 true"
    );

    let row = registry
        .get::<AbilityConfig>(&2)
        .expect("主索引 id=2 应命中");
    assert_eq!(row.id, 2);
    assert_eq!(row.name, "ice");
    assert_eq!(row.primary_key(), 2, "primary_key() 应提取主索引字段值");

    assert!(
        registry.get::<AbilityConfig>(&999).is_none(),
        "已加载表的不存在键应返回 None"
    );
}

/// §5.4「单表生命周期」+ B2-4「unload 后失效」：unload 后 is_loaded false、get 返回 None。
#[test]
fn unload_then_get_returns_none() {
    let mut registry = DataRegistry::default();
    registry.insert(ability_table());
    assert!(registry.is_loaded::<AbilityConfig>(), "insert 后应已加载");

    registry.unload::<AbilityConfig>();

    assert!(
        !registry.is_loaded::<AbilityConfig>(),
        "unload 后 is_loaded 应为 false"
    );
    assert!(
        registry.get::<AbilityConfig>(&1).is_none(),
        "unload 后的查询必须返回 None（引用失效，B2-4）"
    );
}

/// §5.4「重复 load 幂等性」（registry 级对应）：同类型表重复 insert = 替换——
/// 查询命中**最新**数据，无累积、无报错（load 内部即 insert，重复 load 不应叠加表）。
#[test]
fn repeated_insert_replaces_previous_table() {
    let mut registry = DataRegistry::default();

    registry.insert(ability_table());

    let updated = DataTable::from_rows(vec![
        AbilityConfig {
            id: 1,
            name: "fireball-v2".to_string(),
        },
        AbilityConfig {
            id: 2,
            name: "ice".to_string(),
        },
        AbilityConfig {
            id: 3,
            name: "thunder".to_string(),
        },
        AbilityConfig {
            id: 4,
            name: "blizzard".to_string(),
        },
    ])
    .expect("合法数据构建索引不应失败");
    registry.insert(updated);

    assert!(
        registry.is_loaded::<AbilityConfig>(),
        "重复 insert 后仍应保持加载态"
    );
    let row = registry
        .get::<AbilityConfig>(&1)
        .expect("主索引 id=1 应命中");
    assert_eq!(
        row.name, "fireball-v2",
        "重复 insert 应以最新表为准（替换而非累积）"
    );
    assert!(
        registry.get::<AbilityConfig>(&4).is_some(),
        "仅存在于新表的主键应可查询——证明旧表已被替换而非叠加"
    );
}

/// 盲区探测（D4 TypeId 擦除）：不同类型表在 registry 中互不可见——
/// 查询与生命周期按行类型隔离，互不干扰。
#[test]
fn tables_of_different_types_are_isolated() {
    let mut registry = DataRegistry::default();

    let row = NoIndexRow {
        id: 10,
        name: "ten".to_string(),
    };
    assert_eq!(row.id, 10, "构造无索引行字段读取");
    registry.insert(DataTable::from_rows(vec![row]).expect("无索引表构建不应失败"));

    assert!(
        !registry.is_loaded::<AbilityConfig>(),
        "插入 NoIndexRow 不应影响 AbilityConfig 的加载态"
    );
    assert!(registry.get::<AbilityConfig>(&10).is_none());

    registry.insert(ability_table());
    assert!(registry.is_loaded::<AbilityConfig>());
    assert!(registry.get::<AbilityConfig>(&1).is_some());

    registry.unload::<AbilityConfig>();
    assert!(!registry.is_loaded::<AbilityConfig>());
    assert!(
        registry.is_loaded::<NoIndexRow>(),
        "unload AbilityConfig 不应影响 NoIndexRow 的加载态"
    );
}

/// 盲区探测（Batch 2 设计「无索引行类型 → `PrimaryKey = ()`」）：无索引表可存储、
/// 生命周期可管理——insert/is_loaded/unload 全链路工作；`primary_key()` 返回 `()`。
/// （不锁定 `get::<T>(&())` 语义——无主索引的表按主键查询行为 spec 未定义、未要求。）
#[test]
fn no_index_table_is_storable_and_lifecycle_managed() {
    let mut registry = DataRegistry::default();

    let row = NoIndexRow {
        id: 1,
        name: "a".to_string(),
    };
    assert_eq!(row.primary_key(), (), "无索引行类型的 PrimaryKey 应为 ()");
    registry.insert(DataTable::from_rows(vec![row]).expect("无索引表构建不应失败"));
    assert!(registry.is_loaded::<NoIndexRow>());

    registry.unload::<NoIndexRow>();
    assert!(!registry.is_loaded::<NoIndexRow>());
}
