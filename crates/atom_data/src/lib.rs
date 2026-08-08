//! atom_data — 声明式数据表框架（bevy_common_assets 驱动，全面替代 Luban 二进制 datatables 体系）。
//!
//! 权威 spec：`.omo/plans/atom-data.md`（issue #3 Batch 1）。
//! 关键决策：D1（行类型 + `DataTable<T>` 泛型表容器）、D2（`DataIndexed`/`DataIndex` 索引系统）、
//! Q2/Q3/Q10（bevy_common_assets 全格式、目录约定 `assets/datatables/<表类型名>.json`）。
//!
//! # RED 阶段占位
//!
//! 本 crate 尚**未实现**。集成测试 `tests/index.rs`、`tests/deserialize.rs` 引用了以下公共 API
//! 形态——当前全部无法解析，**编译失败即"实现缺失"的干净信号**（测试先行，主 agent 待 GREEN）：
//!
//! - `DataAsset` derive 宏：行类型 → `DataIndexed` impl，解析 `#[index(...)]` 属性
//! - `DataIndexed` trait：`type Index: DataIndex<Self>`（D2，宏生成 HashMap 族容器）
//! - `DataTable<T: DataIndexed>` 泛型表：`rows: Vec<T>` + `T::Index`，`from_rows` + `Deserialize`
//!   两条构建路径均须构建索引（D1 / B1-3）

#![deny(missing_docs)]
