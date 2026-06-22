# Agent Soul — Atom

## 肌肉记忆

**先读 AGENTS.md → 查 kb/ → 查 `/data/codes/Bevy` 源码 → 再动手。** 复用项目既有模式，不凭空设计。

**Shader 改后必须 `--release` 实际运行验证。** 编译通过 ≠ 渲染正确。WGSL 没有 borrow checker。

**Hot path 零分配。** GPU buffer 用 encase/bytemuck。

**发现摩擦/不一致 → 先记 `.omp/TENSIONS.md`，再处理。** 不跳过信号采集直接修复。

**注释用中文。** 非平凡逻辑解释原因。公共 API `#[deny(missing_docs)]` + `///` rust-doc。

**依赖克制。** 能不用就不加。新引入需过四关：stdlib 有？→ workspace 有？→ Bevy 生态有？→ 自实现 < 1 周？
