# SESSION LOG

## 2026-06-20 — 全量重写 + 流程加固

- 删除全部旧地形代码（biome/LOD/CSG/isosurface/default_compute）
- Bevy 0.19 适配，本地仓库 `/data/codes/Bevy` checkout v0.19.0
- 补 `#![feature(cfg_select)]` 到 bevy_app + bevy_winit
- 新架构：四 pass GPU compute (density→cross→QEF→indices)，固定 slot 顶点/索引
- 更新 workflow.lisp：加 Phase-Gate Protocol、document phase、anti-patterns
- 强制 `#[deny(missing_docs)]` + 全量 rust-doc (RFC 1574)
- Spec 文件重写为 MVP 现状，LOD spec 删除
- 合并 `.plan/` 和 `specs/` 到 `.claude/plan/` 和 `.claude/specs/`
- 所有引用路径同步更新

## 2026-06-19 — 工作流初始化

- 创建 `.claude/`：CLAUDE.md, AGENTS.md, APPEND_SYSTEM.md, SOUL.md, CAPABILITY-MAP.md
- 创建 `specs/`：vision, terrain-shape, terrain-material, lod
- 创建 `.plan/`：PLAN, MEMORY, DRIFT, SESSION-LOG
- 初始化 TENSIONS.md
- 归档旧 `doc/` 至 `doc/_archive/`
- 确立「程序化优先」方向：CSG 编辑降级，LOD 提升为 Phase 4

**Handoff:**
- Next: Phase 1 — Rust MVP smoke test（`cargo run --example chunk_loader`）
- 密度场：value noise 单 biome，地表 = y - height_at(x,z)
- Workspace: 仅 `crates/atom_terrain`
