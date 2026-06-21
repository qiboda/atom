# MEMORY — 架构决策记录

| # | Date | Decision | Rationale |
|---|------|----------|-----------|
| 1 | 2025 | GPU 端 Dual Contouring | CPU 无法实时处理大规模体素 |
| 2 | 2025 | QEF (Cramer's rule) | 3×3 矩阵求解简洁，退化时降级为平均 |
| 3 | 2025 | WGSL | Bevy + wgpu 原生；跨平台；热重载 |
| 4 | 2025 | crossbeam render→main | Bevy render world 隔离；异步传输 |
| 5 | 2025 | mod.rs 模块组织 | Rust 2018+ 标准 |
| 6 | 2025 | expect 替代 unwrap | Clippy 强制 |
| 7 | 2026-06 | 程序化优先 | 算法生成 > 手工编辑；biome 后置 |
| 8 | 2026-06 | Bevy 0.19 + 本地仓库 patch | 最新 API（RenderContext/BindGroupLayoutEntries），本地可调试 |
| 9 | 2026-06 | 固定 slot 顶点/索引（无 atomics） | MVP 简洁，atom 类型匹配问题规避 |
| 10 | 2026-06 | `#[deny(missing_docs)]` + rust-doc RFC 1574 | 强制文档先于实现，spec 与 doc 分工 |
| 11 | 2026-06 | Phase-Gate Protocol | `[GATE from→to]` 声明防止阶段跳过 |
| 12 | 2026-06 | 2D value noise (MVP) | 简化实现，后续替换为 OpenSimplex |
| 13 | 2026-06-21 | Agent Sidecar: TypeScript + BRP | 游戏逻辑脚本化，零编译迭代；BRP 自动暴露 #[reflect] 组件，零绑定；Agent 独立进程，崩溃不传染引擎 |
| 14 | 2026-06-21 | NPC 可视化：Bevy 侧 decorate 系统 | BRP spawn_entity 无法创建 Mesh/Material handle；Agent spawn 带 Name("NPC")，Bevy Update 系统检测后补 cube+material |
| 15 | 2026-06-21 | Agent 生命周期：Drop trait | cleanup_agent 在 Last schedule 每帧触发 → 首帧 kill agent。改用 Resource Drop 在 App 退出时自动 kill |
