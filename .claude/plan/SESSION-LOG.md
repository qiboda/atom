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

## 2026-06-20 — mesh 形状稳定化

- **根因**: `edge_detect.wgsl` 密度采样 `round()` 量化 → 阶梯函数二分搜索 → 交叉点锁在体素边界
- **修复 1**: `grid_idx_from_world` → `trilinear_sample()` 三线性插值（全局 + per-chunk 两个 shader）
- **修复 2**: QEF 求解加 Probabilistic Quadrics 正则化 `A += ncross·σ²I`, `b += σ²·Σp`（全局 + per-chunk 两个 shader）
- **修复 3**: grid_min world-aligned（对齐 grid_size*voxel_size=25m 边界），不再跟 observer 0.5m 移动
- **附带**: `.omo/` work plan 整合进 `.claude/plan/completed/`
- **验证**: `cargo check` / `cargo clippy` / `cargo test` 全绿，runtime smoke test 通过 (3450 verts, 6664 tris)
- **Handoff**: `feature/gpu-mesh-seams` 分支就绪；下一步考虑 terrain shader 直接 `#import quadric` 消除 atom_pqef 重复实现

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
