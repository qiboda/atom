# Atom 项目铁律

- 禁止 `unwrap()` → 统一使用 `expect("原因")`
- 公共 API 强制 `#[deny(missing_docs)]` + rust-doc (RFC 1574)
- Bevy 运行/测试必须用 `--release`（debug 极慢）
- 不使用 `thiserror`/`anyhow`
- GPU mesh 仅用于渲染，不参与物理/寻路
- 不碰 `.atom.project` 标记文件
