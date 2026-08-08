//! 原子表类型（serde + DataAsset 声明，B3-1）——替代 Luban 生成的 atom_datatables 表。
//!
//! 字段与 Luban Bean（`atom_datatables/gen/atom_cfg/src/effect.rs` 的 Ability/Buff、
//! `layertag.rs` 的 LayerTag）一致，但：
//! - 表类型改为 `#[derive(DataAsset)]` 的行类型 + `DataTable<T>` 泛型表容器（D1）；
//! - 主索引声明化：`AbilityConfig`/`BuffConfig` 主索引 `id`，`LayerTagConfig` 主索引
//!   `raw_layertag`（**String 主键**——现有跨表引用全是 String 键，D3 锁定）；
//! - 4 个 `Vec<String>` layertag 字段声明 `#[data_ref(table, key)]` 跨表引用（B3-4），
//!   宏生成 `resolve_{field}` 惰性解析；`start_added/removed_layertags:
//!   Vec<RevertableLayerTag>` 是内嵌结构（非键列表），不声明 data_ref；
//! - `AbilityType` 由 Luban `From<i32>` repr 枚举迁移为 serde 单元枚举
//!   （JSON 字符串形态 `"Active"`/`"Passive"`）；
//! - `RevertableLayerTag` 在本模块重定义（不再从 atom_datatables 导入）。

use atom_data::DataAsset;

/// 技能激活类型（serde 单元枚举，JSON 字符串形态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum AbilityType {
    /// 主动技能。
    Active,
    /// 被动技能。
    Passive,
}

/// 可回滚状态层标签（内嵌结构，非跨表引用）。
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct RevertableLayerTag {
    /// 原始标签字符串。
    pub raw_layertag: String,
    /// 是否可回滚。
    pub revertable: bool,
}

/// 技能配置表行（主索引 `id`，字段与 Luban Ability Bean 一致）。
#[derive(serde::Deserialize, DataAsset, Debug, Clone, PartialEq)]
#[index(key = "id")]
pub struct AbilityConfig {
    /// 技能 id（主索引）。
    pub id: i32,
    /// 名字。
    pub name: String,
    /// 描述。
    pub desc: String,
    /// 技能图类型名字。
    pub graph_class: String,
    /// 激活类型。
    pub activation_type: AbilityType,
    /// 冷却时间。
    pub cd: f32,
    /// 技能启动所需状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub start_required_layertags: Vec<String>,
    /// 技能启动禁用状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub start_disabled_layertags: Vec<String>,
    /// 技能启动要添加的状态层标签（内嵌结构）。
    pub start_added_layertags: Vec<RevertableLayerTag>,
    /// 技能启动要移除的状态层标签（内嵌结构）。
    pub start_removed_layertags: Vec<RevertableLayerTag>,
    /// 技能中断所需状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub abort_required_layertags: Vec<String>,
    /// 技能中断禁用状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub abort_disabled_layertags: Vec<String>,
}

/// Buff 配置表行（主索引 `id`，字段与 Luban Buff Bean 一致）。
#[derive(serde::Deserialize, DataAsset, Debug, Clone, PartialEq)]
#[index(key = "id")]
pub struct BuffConfig {
    /// Buff id（主索引）。
    pub id: i32,
    /// 名字。
    pub name: String,
    /// 描述。
    pub desc: String,
    /// 技能图类型名字。
    pub graph_class: String,
    /// 最大层数。
    pub max_layer: i32,
    /// 时长。
    pub duration: f32,
    /// 间隔。
    pub interval: f32,
    /// 技能启动所需状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub start_required_layertags: Vec<String>,
    /// 技能启动禁用状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub start_disabled_layertags: Vec<String>,
    /// 技能启动要添加的状态层标签（内嵌结构）。
    pub start_added_layertags: Vec<RevertableLayerTag>,
    /// 技能启动要移除的状态层标签（内嵌结构）。
    pub start_removed_layertags: Vec<RevertableLayerTag>,
    /// 技能中断所需状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub abort_required_layertags: Vec<String>,
    /// 技能中断禁用状态层标签（跨表引用 LayerTagConfig 主键）。
    #[data_ref(table = "LayerTagConfig", key = "raw_layertag")]
    pub abort_disabled_layertags: Vec<String>,
}

/// 状态层标签配置表行（主索引 `raw_layertag` **String 主键**，字段与 Luban LayerTag Bean 一致）。
#[derive(serde::Deserialize, DataAsset, Debug, Clone, PartialEq)]
#[index(key = "raw_layertag")]
pub struct LayerTagConfig {
    /// 原始标签字符串（主索引，以 `.` 作为分隔符）。
    pub raw_layertag: String,
    /// 描述。
    pub desc: String,
    /// 是否计数。
    pub counter: bool,
}
