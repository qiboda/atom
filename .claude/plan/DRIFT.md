# DRIFT — 规格偏离追踪

| # | Date | What Changed | Why | Spec Updated? |
|---|------|-------------|-----|---------------|
| 1 | 2026-06-20 | 删除全部旧代码（biome/LOD/CSG/isosurface），Bevy 0.19 重写 | 旧代码质量差，API 过时，需求简化 | Yes |
| 2 | 2026-06-20 | Biomes 推迟到最后做 | 单 biome MVP 优先，先跑通 GPU 管线 | Yes |
| 3 | 2026-06-20 | Workspace 仅保留 atom_terrain | 其他 crate 未升级到 0.19，移出 workspace | Cargo.toml |
| 4 | 2026-06-20 | GPU staging buffer readback (pass 5-7) | GPU 结果需回传 CPU 组装 Mesh | terrain-shape.spec |
| 5 | 2026-06-20 | CPU noise (OpenSimplex2D) ≠ GPU noise (value noise) | CPU 端未同步；两者高度不同（-0.14 vs -26） | terrain-shape.spec C4 ⚠️ |
| 6 | 2026-06-20 | QEF 顶点外溢 → CPU 端 clamp 到 chunk bounds | value noise 尖锐梯度 | terrain-shape.spec |
