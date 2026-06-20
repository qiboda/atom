# DRIFT — 规格偏离追踪

| # | Date | What Changed | Why | Spec Updated? |
|---|------|-------------|-----|---------------|
| 1 | 2026-06-20 | 删除全部旧代码（biome/LOD/CSG/isosurface），Bevy 0.19 重写 | 旧代码质量差，API 过时，需求简化 | Yes (project.spec, terrain-shape.spec, terrain-material.spec) |
| 2 | 2026-06-20 | Biomes 推迟到最后做 | 单 biome MVP 优先，先跑通 GPU 管线 | Yes (project.spec, terrain-shape.spec) |
| 3 | 2026-06-20 | Workspace 仅保留 atom_terrain | 其他 crate 未升级到 0.19，移出 workspace | Reflected in Cargo.toml members |
