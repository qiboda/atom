# PLAN — GPU DC 地形管线 (MVP → Production)

## Phase 0 — 架构重写 ✅

> 2026-06-20: 删除全部旧代码（biome/LOD/CSG/isosurface），Bevy 0.19 重写。

- [x] 0.1 工作区精简：仅保留 `atom_terrain` crate
- [x] 0.2 Bevy 0.19 适配（本地仓库 `/data/codes/Bevy` v0.19.0）
- [x] 0.3 四 pass GPU compute pipeline（density→cross→QEF→indices）
- [x] 0.4 固定 slot 顶点/索引 + CPU compact remap
- [x] 0.5 ExtractResource 同步 main→render chunk 队列
- [x] 0.6 `#[deny(missing_docs)]` + 全量 rust-doc
- [x] 0.7 Phase-Gate Protocol + document phase in workflow.lisp

Exit: `cargo check` / `cargo clippy` / `cargo doc` / `cargo test` 通过。

## Phase 1 — Rust MVP smoke test ✅

> Smoke: GPU 端到端验证 shader 编译 + mesh 生成 + 渲染。

- [x] 1.1 运行 `cargo run -p atom_terrain --example chunk_loader`
- [x] 1.2 验证 shader 编译通过（无 WGSL compiler error）
- [x] 1.3 验证 CPU noise 与 GPU noise 结果一致
- [x] 1.4 单 chunk mesh 正确渲染（GPU→CPU staging buffer readback 已实现）

Exit: 可见绿色地形 mesh。✅ GPU readback 完成，mesh 通过 crossbeam 回传主世界渲染。

## Phase 2 — 多 chunk 动态加载 ✅

> Spec: `.claude/specs/terrain-shape.spec`

- [x] 2.1 `TerrainObserver` + `update_grid_chunks` 功能验证
- [x] 2.2 多 chunk 同时 compute（buffer pool 复用）
- [x] 2.3 QEF 确定性边界无缝验证

Exit: 移动摄像机可见连续无缝的多 chunk 地形。✅ 100 chunk 动态加载，双边 shell + fallback 顶点填补边界缝隙。

## 后续储备

不在当前 plan 中，方向定义在 `vision.md`：

- Biome 分布与纹理
- TerrainMaterial（biome 驱动 PBR）
- 寻路 (nav mesh)
- 技能系统与地形交互
- CSG 洞穴/构造物生成（系统驱动，非玩家编辑）
- **CI 冒烟测试**: `chunk_loader` 启动 → 截图 → 与基准截图 diff。可自动捕获 mesh 几何回归。需确定截图方案（headless render / PIX / RenderDoc）。
