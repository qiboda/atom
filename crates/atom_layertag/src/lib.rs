#![deny(missing_docs)]
//! # atom_layertag
//!
//! 图层标签（LayerTag）系统 crate：以点分路径（如 `a.b.c`）描述实体/能力的归属层级，
//! 提供标签定义、注册表与容器操作。
//!
//! 核心概念：
//! - [`tag::Tag`]：标签片段——图层标签路径中的单个节点。
//! - [`layertag::LayerTag`]：图层标签——多个 [`tag::Tag`] 以 `.` 连接组成的完整路径。
//! - [`registry::LayerTagRegistry`]：注册表——登记合法图层标签，保证请求到的标签均已注册。
//! - [`container_op`]：容器抽象——增删查接口 [`container_op::LayerTagContainer`]，
//!   以及可组合的操作 [`container_op::LayerTagContainerOp`] 与条件 [`container_op::LayerTagContainerCondition`]。
//! - [`single_container`] / [`count_container`]：去重 / 带引用计数的容器实现。
//! - [`builder`]：图层标签的链式构建器。

/// [`LayerTag`](layertag::LayerTag) 构建器。
pub mod builder;
/// 图层标签容器的抽象操作与条件。
pub mod container_op;
/// 带引用计数的图层标签容器。
pub mod count_container;
/// 带引用计数的图层标签。
pub mod count_layertag;
/// 图层标签本体。
pub mod layertag;
/// 图层标签注册表。
pub mod registry;
/// 去重的图层标签容器。
pub mod single_container;
/// 标签片段。
pub mod tag;
