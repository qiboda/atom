//! data_ref 跨表引用集成测试（纯逻辑，不启动 Bevy App）。
//!
//! spec 依据：`.dsh/plans/atom-data.md` §5（issue #4 Batch 2）——
//! - §5.3 验收「`#[data_ref]` 字段生成解析方法，目标表未加载返回 None」
//! - §5.4「两表跨表引用：引用方先加载、被引用方后加载 → 解析仍成功（惰性解析时机）」
//! - §5.4「data_ref 键不存在 → 跳过/None（行为实现时定，测试锁定 skip）」
//! - B2-3「load 顺序无关（先/后加载均能解析）」、B2-4「unload 后失效」
//! - 关键决策 D3（字段保持原始类型 `Vec<String>` = 目标表键列表，不侵入序列化格式；
//!   宏生成惰性解析方法，目标表未加载返回 None）
//!
//! ## 从 spec 推断的 API 契约（RED 锁定，实现必须遵守）
//!
//! 1. `#[data_ref(table = "LayerTagConfig", key = "id")]` 是 `DataAsset` derive 的
//!    helper attribute（D3）：字段保持 `Vec<String>`（目标表主键的字符串形式），
//!    serde 序列化格式不受影响。
//! 2. 宏生成解析方法 `resolve_{field}`：`resolve_start_required_layertags(&self,
//!    registry: &DataRegistry) -> Option<Vec<&LayerTagConfig>>`。
//! 3. **惰性语义**（Q8）：目标表未加载 → `None`；已加载 → 逐 key 解析返回
//!    `Vec<&LayerTagConfig>`，**保持引用列表顺序**。
//! 4. **键不存在 → 跳过**（返回存在子集，不 panic）；目标表已加载但全部键缺失 →
//!    `Some(空)`（区别于未加载的 `None`）。
//! 5. **unload 失效**：目标表 unload 后再次解析 → `None`（B2-4，与「未加载 → None」
//!    语义一致）。
//! 6. 解析返回的 `&LayerTagConfig` 借自 registry（目标行数据在 registry 内）——输出
//!    生命周期必须绑定 **registry 参数**，不能依赖 `&self` 的 elision。
//! 7. 键类型转换：字段元素是 `String`（目标表主键 `id: i32` 的字符串形式），解析时
//!    转换后按目标表主索引查询（D3「String 键 + 惰性解析」兼容语义）。

use atom_data::{DataAsset, DataRegistry, DataTable};

/// 被引用表：主索引 id（data_ref 的 `key = "id"` 指向该主索引）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
struct LayerTagConfig {
    id: i32,
    raw_layertag: String,
}

/// 引用方：`#[data_ref(table = "LayerTagConfig", key = "id")]` 声明跨表引用，
/// 字段保持原始类型 `Vec<String>`（= 目标表主键列表的字符串形式，D3）。
#[derive(serde::Deserialize, DataAsset)]
#[index(key = "id")]
struct AbilityConfig {
    id: i32,
    name: String,
    #[data_ref(table = "LayerTagConfig", key = "id")]
    start_required_layertags: Vec<String>,
}

fn layer_tag_table() -> DataTable<LayerTagConfig> {
    DataTable::from_rows(vec![
        LayerTagConfig {
            id: 1,
            raw_layertag: "fire".to_string(),
        },
        LayerTagConfig {
            id: 2,
            raw_layertag: "ice".to_string(),
        },
        LayerTagConfig {
            id: 3,
            raw_layertag: "thunder".to_string(),
        },
    ])
    .expect("合法数据构建索引不应失败")
}

/// 引用列表 ["1", "3"] → 目标表主键 1、3。
fn ability_table() -> DataTable<AbilityConfig> {
    DataTable::from_rows(vec![AbilityConfig {
        id: 1,
        name: "fireball".to_string(),
        start_required_layertags: vec!["1".to_string(), "3".to_string()],
    }])
    .expect("合法数据构建索引不应失败")
}

/// §5.3 验收「目标表未加载返回 None」+ Q8 惰性：仅引用方表已加载、被引用方未加载 →
/// 解析返回 None。
#[test]
fn resolve_returns_none_when_target_not_loaded() {
    let mut registry = DataRegistry::default();
    registry.insert(ability_table());
    // 注意：LayerTagConfig 表刻意未插入

    let ability = registry
        .get::<AbilityConfig>(&1)
        .expect("引用方表已加载应命中");
    assert!(
        ability
            .resolve_start_required_layertags(&registry)
            .is_none(),
        "目标表未加载时解析必须返回 None（惰性，Q8）"
    );
}

/// D3 解析语义：两表均已加载 → 逐 key 解析返回 `Vec<&LayerTagConfig>`，
/// 保持引用列表顺序（["1", "3"] → id 1、id 3）。
#[test]
fn resolve_returns_matched_rows_in_field_order() {
    let mut registry = DataRegistry::default();
    registry.insert(ability_table());
    registry.insert(layer_tag_table());

    let ability = registry
        .get::<AbilityConfig>(&1)
        .expect("引用方表已加载应命中");
    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some");

    assert_eq!(resolved.len(), 2, "引用列表 2 个键应解析出 2 行");
    assert_eq!(resolved[0].id, 1, "应按引用列表顺序解析");
    assert_eq!(resolved[0].raw_layertag, "fire");
    assert_eq!(resolved[1].id, 3, "应按引用列表顺序解析");
    assert_eq!(resolved[1].raw_layertag, "thunder");
}

/// §5.4「引用方先加载、被引用方后加载 → 解析仍成功（惰性解析时机）」：
/// 解析是调用时的惰性查找（Q8），与加载顺序无关——目标表加载完成后解析成功。
#[test]
fn resolve_succeeds_when_referencer_loaded_before_target() {
    let mut registry = DataRegistry::default();
    registry.insert(ability_table());

    // 目标表尚未加载 → None
    {
        let ability = registry
            .get::<AbilityConfig>(&1)
            .expect("引用方表已加载应命中");
        assert!(
            ability
                .resolve_start_required_layertags(&registry)
                .is_none(),
            "目标表加载前解析应为 None"
        );
    }

    // 后加载目标表（顺序无关：引用方先、被引用方后）
    registry.insert(layer_tag_table());

    let ability = registry
        .get::<AbilityConfig>(&1)
        .expect("引用方表已加载应命中");
    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("目标表加载完成后解析应成功");
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].id, 1);
    assert_eq!(resolved[1].id, 3);
}

/// B2-3「load 顺序无关」反向：被引用方先加载、引用方后加载 → 解析仍成功。
#[test]
fn resolve_succeeds_when_target_loaded_before_referencer() {
    let mut registry = DataRegistry::default();
    registry.insert(layer_tag_table());

    registry.insert(ability_table());

    let ability = registry
        .get::<AbilityConfig>(&1)
        .expect("引用方表已加载应命中");
    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("两表均已加载，解析应成功");
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].raw_layertag, "fire");
    assert_eq!(resolved[1].raw_layertag, "thunder");
}

/// §5.4「data_ref 键不存在 → 跳过」（锁定 skip）：引用列表含不存在于目标表的键
/// （"999"）→ 跳过，返回存在子集，不 panic。
#[test]
fn resolve_skips_keys_missing_in_target() {
    let mut registry = DataRegistry::default();
    registry.insert(layer_tag_table());

    // 引用方行直接构造（解析是行级惰性操作，只读自身字段 + 目标表状态）
    let ability = AbilityConfig {
        id: 1,
        name: "fireball".to_string(),
        start_required_layertags: vec!["1".to_string(), "999".to_string(), "3".to_string()],
    };

    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("目标表已加载，解析应返回 Some（即使部分键缺失）");

    assert_eq!(resolved.len(), 2, "缺失键 \"999\" 应被跳过");
    assert_eq!(resolved[0].id, 1);
    assert_eq!(resolved[1].id, 3);
}

/// 盲区探测：目标表已加载但**全部**引用键缺失 → `Some(空 Vec)`——「跳过」语义在
/// 全部缺失时退化为空结果集，**不是** `None`（`None` 仅表示目标表未加载）。
#[test]
fn resolve_all_missing_keys_yields_some_empty_when_table_loaded() {
    let mut registry = DataRegistry::default();
    registry.insert(layer_tag_table());

    let ability = AbilityConfig {
        id: 1,
        name: "fireball".to_string(),
        start_required_layertags: vec!["999".to_string(), "888".to_string()],
    };

    let resolved = ability
        .resolve_start_required_layertags(&registry)
        .expect("目标表已加载：全部键缺失也应返回 Some");
    assert!(
        resolved.is_empty(),
        "全部键缺失 → 跳过全部 → 空结果集（仍为 Some，与未加载的 None 区分）"
    );
}

/// §5.4 / B2-4「解析后目标表被 unload → 引用失效」：目标表 unload 后再次解析返回
/// None（锁定 None——unload 后目标表等同未加载，与 Q8 惰性语义一致）。
#[test]
fn resolve_returns_none_after_target_unloaded() {
    let mut registry = DataRegistry::default();
    registry.insert(ability_table());
    registry.insert(layer_tag_table());

    // 目标表加载期间：解析成功
    {
        let ability = registry
            .get::<AbilityConfig>(&1)
            .expect("引用方表已加载应命中");
        assert!(
            ability
                .resolve_start_required_layertags(&registry)
                .is_some(),
            "目标表加载期间解析应成功"
        );
    } // 块结束：释放全部 registry 借用，随后才能可变借用 unload

    registry.unload::<LayerTagConfig>();

    assert!(
        registry.is_loaded::<AbilityConfig>(),
        "unload 目标表不应影响引用方表"
    );
    let ability = registry
        .get::<AbilityConfig>(&1)
        .expect("引用方表已加载应命中");
    assert!(
        ability
            .resolve_start_required_layertags(&registry)
            .is_none(),
        "目标表 unload 后引用必须失效（返回 None）"
    );
}
