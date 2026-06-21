# Atom 编码铁律

## 错误处理

- 禁止 `unwrap()` → 统一使用 `expect("原因")`
- `panic!` 仅允许硬停止场景
- 可恢复场景用 `warn!` 记录并跳过
- 不使用 `thiserror`/`anyhow`

## 注释与文档

- 非平凡逻辑用中文注释；简单辅助函数用英文；Shader 中英文混合
- 公共 API 强制 `#[deny(missing_docs)]` + `///` rust-doc（RFC 1574: Summary/Examples/Panics/Safety）

## 模块组织

- `mod.rs` 模式；相关 component/system/resource 分组

## 格式化与 Lint

- `rustfmt.toml`: Unix 换行, field init shorthand, edition 2024
- Clippy: 允许 `too_many_arguments`/`type_complexity`/`collapsible_if`；警告 `unwrap_used`

## 构建

- Bevy debug 构建极慢，运行/测试必须用 `--release`
- 验证命令: `cargo run -p atom_terrain --example chunk_loader --release`
