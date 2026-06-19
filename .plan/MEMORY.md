# MEMORY — 架构决策记录

| # | Date | Decision | Rationale |
|---|------|----------|-----------|
| 1 | 2025 | GPU 端 Dual Contouring | CPU 无法实时处理大规模体素 |
| 2 | 2025 | PQEF (Trettner 2020) | 退化配置下更鲁棒 |
| 3 | 2025 | WGSL | Bevy + wgpu 原生；跨平台；热重载 |
| 4 | 2025 | crossbeam render→main | Bevy render world 隔离；异步传输 |
| 5 | 2025 | Voronoi 图生成 biome | 不规则自然形状；种子点可扩展 |
| 6 | 2025 | 顶点 weight blending | 随机地形无法预烘焙 splat map |
| 7 | 2025 | mod.rs 模块组织 | Rust 2018+ 标准 |
| 8 | 2025 | expect 替代 unwrap | Clippy 强制 |
| 9 | 2026-06 | 程序化优先 | 算法生成 > 手工编辑；CSG 降级为系统工具 |
| 10 | 2026-06 | LOD 优先于 CSG | 可视距离直接提升探索体验，CSG 是锦上添花 |
