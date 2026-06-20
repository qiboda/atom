# TENSIONS.md — Atom 项目摩擦日志

> 发现数据与系统设计之间的不一致、工具链问题、或流程阻碍时记录。不要当场解决——只捕获信号。

## GPU 管线

- **2026-06-20**: ✅ 四 pass compute 管线 smoke test 通过 (RTX 4090, Vulkan), 0 GPU validation error。Readback 待实现 (需 staging buffer 方案)。
- **2026-06-20**: WGSL atomics 类型匹配问题 — `array<atomic<u32>>` 在 shader 端需要匹配 Rust 端 binding type 声明。当前绕过（固定 slot 无 atomics）。
- **2026-06-20**: ExtractResourcePlugin 在 Bevy 0.19 pipelined rendering 中不工作 → render world 资源必须直接用 `render_app.insert_resource()`。`render_app.init_resource()` 也可。
- **2026-06-20**: WGSL `var<storage>` 无访问修饰符默认 `read`（非 `read_write`）。向 storage buffer 写入必须声明 `var<storage, read_write>`。

## 数据对齐

- **2026-06-20**: 顶点 buffer 使用固定 slot 稀疏存储（每 voxel 一个 TerrainChunkVertex slot），CPU 端 compact + remap。buffer 大小为 vc³ × sizeof(Vertex)，16³=128KB。后续可能需要 compact 存储优化。

## 工具链

- **2026-06-20**: Bevy 0.19 需要 Rust nightly + `#![feature(cfg_select)]`。本地 Bevy checkout v0.19.0 的 `bevy_app` 和 `bevy_winit` 缺失该 feature flag，需手动补丁。
- **2026-06-20**: `cargo doc -Dwarnings` 会把 Bevy 依赖的 warning 也当 error。已限定 `-p atom_terrain` 范围。
- **2026-06-20**: WGSL 无断点 — compute shader 调试全靠肉眼。

## 流程

- **2026-06-20**: intent.lisp、APPEND_SYSTEM.md 全量过时。手工维护的元文档总是落后于代码修改。待方案：自动生成 intent.lisp，APPEND_SYSTEM.md 改为指针索引。
- **2026-06-20**: 旧 specs/lod.spec 已删，specs/terrain-shape.spec 和 specs/terrain-material.spec 重写为 MVP 现状。specs/ → .claude/specs/，.plan/ → .claude/plan/。
- **2026-06-20**: Workspace 仅保留 atom_terrain。其他 14 个 crate 暂时移出，待逐步迁入 Bevy 0.19。
- **2026-06-20**: Workflow 加 document phase + bevy-kb 更新 + verify-references 检查。Phase-Gate Protocol 防止阶段跳过。

## 已知退化

- 密度场使用简版 value noise（非 OpenSimplex），视觉质量低于目标。
- 使用 StandardMaterial（单色绿色），非 biome 驱动 PBR。
- 无 LOD — 所有 chunk 用相同 16³ 分辨率。
- 无 biome — 所有 chunk 地形形状相同。
