#![deny(missing_docs)]
//! Atom 引擎核心工具库：提供统一的日志初始化（`atom_log_plugin`）与项目路径解析（`ProjectPaths`）。

/// 日志插件模块：基于 `tracing` 的文件日志输出。
pub mod logger;
/// 项目路径模块：通过 `.atom.project` 标记文件定位项目根目录。
pub mod paths;
