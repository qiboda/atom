# TENSIONS.md — Atom 项目摩擦日志

> 发现数据与系统设计之间的不一致、工具链问题、或流程阻碍时记录。不要当场解决——只捕获信号。

## GPU 管线

- **2026-06-20**: ✅ 四 pass compute 管线 smoke test 通过 (RTX 4090, Vulkan), 0 GPU validation error。Readback 待实现 (需 staging buffer 方案)。
- **2026-06-20**: WGSL atomics 类型匹配问题 — `array<atomic<u32>>` 在 shader 端需要匹配 Rust 端 binding type 声明。当前绕过（固定 slot 无 atomics）。
- **2026-06-20**: ExtractResourcePlugin 在 Bevy 0.19 pipelined rendering 中不工作 → render world 资源必须直接用 `render_app.insert_resource()`。`render_app.init_resource()` 也可。
- **2026-06-20**: WGSL `var<storage>` 无访问修饰符默认 `read`（非 `read_write`）。向 storage buffer 写入必须声明 `var<storage, read_write>`。

## 数据对齐

- **2026-06-20**: 顶点 buffer 使用固定 slot 稀疏存储（每 voxel 一个 TerrainChunkVertex slot），CPU 端 compact + remap。buffer 大小为 vc³ × sizeof(Vertex)，16³=128KB。后续可能需要 compact 存储优化。

## Phase 2 接缝修复发现

- **2026-06-20**: 多 chunk 边界有 1-voxel 缝隙 — 根因是边界 voxel 的 quad 引用的邻接 voxel slot 不存在。修复：双边 shell（density (vc+3)³, vertex/cross (vc+2)³）。
- **2026-06-20**: ExtractResourcePlugin 在 Bevy 0.19 sub-app 中不工作 → main→render 同步改用 crossbeam channel（`ChunkProcessRequest` + Sender/Receiver）。
- **2026-06-20**: QEF Cramer's rule 对平面退化 — 行列式阈值 `1e-10` 太小，f32 近奇异矩阵产生 spike。修到 `1e-5` + centroid 安全网（顶点距 centroid > 2×voxel_size 则回落）。
- **2026-06-20**: 每个 voxel 只有 6 个 index slot（1 quad）→ 倾斜面上多 quad 互相覆盖导致整行缺失。扩到 72 slot（12 edge × 6 index）。
- **2026-06-20**: `ncross==0` 的边界 voxel 无顶点 → has_vertex 失败 → quad 丢弃。加 fallback 中心点占位。
- **2026-06-20**: Staging buffer 重复 `map_async` panic — 加 `map_started` 标志防止重入。
- **2026-06-20**: 负 shell voxel 顶点为 fallback（无表面交叉）时，邻居无 mesh，不应 skip 该 quad。智能 dedup：仅当负 shell 顶点有真实法线（`length(normal)>0` 表示邻居有 mesh）才跳过。
- **2026-06-20**: Bevy 0.19 内置 `FreeCamera` (`bevy_camera_controller` + `free_camera` feature)。操作：右键旋转 + WASD/QE。
- **2026-06-20**: `StandardMaterial::double_sided` 不管背面剔除 — 需同时设 `cull_mode: None`。

## 工具链

- **2026-06-20**: Bevy 0.19 需要 Rust nightly + `#![feature(cfg_select)]`。本地 Bevy checkout v0.19.0 的 `bevy_app` 和 `bevy_winit` 缺失该 feature flag，需手动补丁。
- **2026-06-20**: `cargo doc -Dwarnings` 会把 Bevy 依赖的 warning 也当 error。已限定 `-p atom_terrain` 范围。
- **2026-06-20**: WGSL 无断点 — compute shader 调试全靠肉眼。

## 流程

- **2026-06-20**: intent.lisp、APPEND_SYSTEM.md 全量过时。手工维护的元文档总是落后于代码修改。待方案：自动生成 intent.lisp，APPEND_SYSTEM.md 改为指针索引。
- **2026-06-20**: 旧 specs/lod.spec 已删，specs/terrain-shape.spec 和 specs/terrain-material.spec 重写为 MVP 现状。specs/ → .omp/specs/，.plan/ → .omp/plan/。
- **2026-06-20**: Workspace 仅保留 atom_terrain。其他 11 个 crate 暂时移出，待逐步迁入 Bevy 0.19。
- **2026-06-20**: Workflow 加 document phase + bevy-kb 更新 + verify-references 检查。Phase-Gate Protocol 防止阶段跳过。

## 已知退化

- 密度场使用简版 value noise（非 OpenSimplex），视觉质量低于目标。GPU value noise height_at(0,0) ≈ -26（CPU OpenSimplex 仅为 -0.14），两者高度不同。
- 使用 StandardMaterial（单色绿色），非 biome 驱动 PBR。
- 无 LOD — 所有 chunk 用相同 16³ 分辨率。
- 无 biome — 所有 chunk 地形形状相同。

## 2026-06-20: readback 实现发现

- **GPU compute 4-pass 管线确认工作**: RTX 4090 Vulkan，17³ 网格 density→cross→QEF→indices 四 pass 输出正确顶点/索引。
- **WGSL binding(5) counters 始终为 0**: 无 atomics 设计下所有 shader 不写 binding 5。vertex_count/index_count 无法从 GPU 读回，需 CPU 端 compact 时自动统计。
- **Staging buffer readback 两帧等待**: dispatch→(1帧等GPU执行)→copy→(1帧等GPU执行)→map→read。同一帧内 dispatch+copy 会读到全零。
- **CPU/GPU 噪声不匹配**: GPU 用 value noise（hash-based），CPU 用 OpenSimplex2D。同一坐标高度不同，无法直接对比验证。测试 chunk 位置需根据 GPU noise 单独定位。
- **2026-06-20**: `edge_detect.wgsl` 密度采样用 `round()` 最近邻 — 将连续密度场量化为阶梯函数，二分搜索收敛到体素边界而非真实 isosurface。根因修复：替换为三线性插值 `trilinear_sample()`。
- **2026-06-20**: `atom_pqef` crate（Rust + WGSL）已有正确的概率 quadric 实现，但 `atom_terrain` 的 terrain shader (`qef_solve.wgsl`, `main_mesh_compute_vertices.wgsl`) 使用独立 inline 实现 — 同一算法两套代码，存在 divergence 风险。已记录分歧原因（性能/复杂度）在 `qef.spec`。

- **2026-06-20**: 回顾发现 `edge_detect.wgsl` 的 `round()` bug 从 Phase 0 存在、多人审阅未发现。根因: 数学密集型函数缺乏 spec 对参。已创建 `.omp/specs/math/` 目录 + `density-sampling.spec` + `qef.spec`。

## 2026-06-21: Agent Sidecar 验收发现

- **2026-06-21**: [agent] `top_down_game.rs` 缺少 `app.run()` — `main()` 构建 App 后直接退出，从未进入游戏循环。exit code 0 无 crash log，难以排查。
- **2026-06-21**: [agent] `cleanup_agent` 注册在 `Last` schedule 每帧运行 — 首帧即 kill agent 进程。修复：改用 `Drop` trait 实现 App 退出时自动清理。
- **2026-06-21**: [agent] BRP `world.spawn_entity` 无法创建 asset handle（Mesh/Material）— NPC 只有 `Transform`，不可见。修复：Agent spawn 时附带 `Name("NPC")`，Bevy 侧 `decorate_agent_entities` 系统自动补 cube mesh + 红色 material。
