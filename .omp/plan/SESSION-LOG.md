# 2026-06-21 — 游戏框架 MVP

## 完成事项

- 讨论并确定整体游戏方向：Agent-as-god 的程序化游戏框架
- 确认核心架构：Bevy engine + BRP (JSON-RPC) ↔ Agent sidecar
- 实现游戏框架 MVP:
  - 俯视角正交摄像机系统（跟随玩家，lerp 平滑）
  - 玩家实体 + WASD 移动
  - ECS 组件体系（Player/Name/Health/MoveSpeed/TopDownCamera，全部 `#[reflect(Component)]`）
  - `top_down_game.rs` example 端到端验证
- 更新 bevy-kb 为新增的三个 Bevy 0.19 API 模式
- 提交 9814956 (8 files +765 -7)

## What went well

- 方向讨论高效，一次对话厘清了 agent 角色、通信协议、初始范围
- 子代理驱动机械性文件创建工作良好，减少了上下文切换
- 设计文档 → 实现计划 → 子代理 → 验证 → 提交，流程完整走了一遍

## What was hard / went wrong

- **Bevy 0.19 API 差异导致两轮编译失败**：`Mesh3d::from()` / `Material3d` / `OrthographicProjection::default()` 都是旧 API 假设。下次应先查 bevy-kb 或 Bevy examples。
- **子代理生成的 example 有重复行**（旧行未清理干净）。子代理写文件后应 read 验证再合并。
- **Edit 工具在代码块内的意外匹配**：编辑计划 markdown 时 SWAP 模式吞掉了代码块内容。编辑 markdown 文件中的代码块要格外小心。
- `PLAN.md` 中的 Git flow 条目也被提交了（之前加的），非本次目标。

## Surprising

- `OrthographicProjection` 不实现 `Default`，而是用 `default_3d()` / `default_2d()` 构造器模式。
- `ScalingMode` 不在 prelude 中，需要 `bevy::camera::ScalingMode` 显式导入。

## Improvements (actionable)

1. **先查 bevy-kb 再写 example**: 本会话中发现 3 个新 API 模式，已写入 patterns.md。下次涉及 Bevy API 时先 grep patterns.md。
2. **子代理产出需验证**: 子代理完成任务后，主 agent 应 read 验证文件内容再合并，避免重复行/遗漏行。
3. **Release build 耗时管理**: LTO + codegen-units=1 编译极慢。验证期用 debug 快速检查，发布前再 release。
4. **思维脑暴提前锁定 Agent 方案**: `bevy_remote` 是 key 基础设施，在方向讨论时尽早确认，避免后续方案翻车。

---
# SESSION LOG

## 2026-06-20 — 全量重写 + 流程加固

- 删除全部旧地形代码（biome/LOD/CSG/isosurface/default_compute）
- Bevy 0.19 适配，本地仓库 `/data/codes/Bevy` checkout v0.19.0
- 补 `#![feature(cfg_select)]` 到 bevy_app + bevy_winit
- 新架构：四 pass GPU compute (density→cross→QEF→indices)，固定 slot 顶点/索引
- 更新 workflow.lisp：加 Phase-Gate Protocol、document phase、anti-patterns
- 强制 `#[deny(missing_docs)]` + 全量 rust-doc (RFC 1574)
- Spec 文件重写为 MVP 现状，LOD spec 删除
- 合并 `.plan/` 和 `specs/` 到 `.omp/plan/` 和 `.omp/specs/`

## 2026-06-20 — mesh 形状稳定化

- **根因**: `edge_detect.wgsl` 密度采样 `round()` 量化 → 阶梯函数二分搜索 → 交叉点锁在体素边界
- **修复 1**: `grid_idx_from_world` → `trilinear_sample()` 三线性插值（全局 + per-chunk 两个 shader）
- **修复 2**: QEF 求解加 Probabilistic Quadrics 正则化 `A += ncross·σ²I`, `b += σ²·Σp`（全局 + per-chunk 两个 shader）
- **修复 3**: grid_min world-aligned（对齐 grid_size*voxel_size=25m 边界），不再跟 observer 0.5m 移动
- **附带**: `.omo/` work plan 整合进 `.omp/plan/completed/`
- **验证**: `cargo check` / `cargo clippy` / `cargo test` 全绿，runtime smoke test 通过 (3450 verts, 6664 tris)
- **Handoff**: 已合并到 `main`；下一步考虑 terrain shader 直接 `#import quadric` 消除 atom_pqef 重复实现

## 2026-06-20 — reflect: mesh 形状稳定化

### What went well
- 三根因追踪链路完整：用户报告 → `round()` 量化 → 奇异 QEF → grid 对齐，全链路修完
- 每个修复改动极小（≤30 行/shaders），但作用互补，组合后 mesh 完全稳定
- `atom_pqef` 已实现概率 quadric，直接对照论文确认正确性，避免重造轮子

### What was hard / went wrong
- **诊断绕了远路**: 前半程花在 wireframe/AABB/command 时序分析上，真正根因是用户一步步指出的。没有从"移动时形状改变"直接跳到"密度采样量化"，而是先被"闪烁"描述误导。**患者描述症状不可靠，应先跑起来看、再测数据，最后猜原因**。
- **编辑工具状态竞争**: `edit` 在多个连续 hunk 时，前一个 edit 导致文件重新编号，后一个用旧 tag 会重新读。连续 edit 应每步用返回的 tag 锚定下一步，而非预读整段。
- **死代码隐患**: `grid_idx_from_world` 的 `round()` 从 Phase 0 就在，多人审阅未发现。**数学密集型函数缺乏 spec 对参**——密度采样行为没有测试/没有文档说明预期。

### Surprising
- `atom_pqef` 和 terrain shader 有两套独立 QEF 实现——Rust 正确，shader 过时。**同算法双实现会散架**（diverging implementations）。
- 用户一句"qef里应该有引用原始论文的标题" + "看看这个"（概率 quadric 论文）比十轮代码阅读更快定位根因。**外部知识引用链比代码内 grep 更有效**。

### Improvements (actionable)
1. **数学函数 spec**: 对 `grid_idx_from_world`/`trilinear_sample`/估计法线/QEF 求解等数学密集型函数，写 mini-spec：输入空间、输出精度、可接受近似误差、边界行为。存在 `.omp/specs/math/` 下。
2. **shader 对照 review**: 新增 shader 时，MUST grep 是否存在同一算法的其他实现（Rust/WGSL/旧 shader），存在则要么统一到一处，要么显式记录 diverged reason。
3. **诊断 protocol**: 对"看起来像 X"的症状描述 → 先量化（录屏/截帧/count 变化频率） → 再隔离变量（关 wireframe / 改 noise / 固定 camera） → 最后下结论。写入 workflow 不变式。
4. **edit 工具纪律**: 每次 edit 必须用上一步返回的 `#TAG`，不得跨 edit 复用旧 tag；连续 edit ≥3 步时中间插入 `read` 确认状态。
5. **CI 冒烟测试**: `chunk_loader` 启动后录第一帧截图，diff 基准截图——可自动捕获 mesh 形状回归。

## 2026-06-19 — 工作流初始化

- 创建 `.omp/`：CLAUDE.md, AGENTS.md, APPEND_SYSTEM.md, SOUL.md, CAPABILITY-MAP.md
- 创建 `specs/`：vision, terrain-shape, terrain-material, lod
- 创建 `.plan/`：PLAN, MEMORY, DRIFT, SESSION-LOG
- 初始化 TENSIONS.md
- 归档旧 `doc/` 至 `doc/_archive/`
- 确立「程序化优先」方向：CSG 编辑降级，LOD 提升为 Phase 4

**Handoff:**
- Next: Phase 1 — Rust MVP smoke test（`cargo run --example chunk_loader`）
- 密度场：value noise 单 biome，地表 = y - height_at(x,z)
- Workspace: 仅 `crates/atom_terrain`
