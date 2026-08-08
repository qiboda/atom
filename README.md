# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形引擎（GPU Dual Contouring + QEF）。

## 安装

```toml
[dependencies]
atom_terrain = { git = "https://github.com/qiboda/atom", branch = "main" }
```

## 使用

```bash
cargo run -p atom_terrain --example chunk_loader
```

## Crate

| Crate | 用途 |
|---|---|
| `atom_terrain` | 核心地形系统 |
| `atom_data` | 声明式数据表框架（DataTable + DataAsset derive） |
| `atom_data_macros` | DataAsset derive 宏（索引系统生成） |
| `atom_pqef` | QEF 求解器 |
| `atom_render` | GPU Buffer |
| `atom_shader_lib` | Shader 工具 |
| `atom_cel_shader` | 赛璐璐渲染 |
| `atom_ability` | 技能系统 |
| `atom_datatables` | 数据表加载（Luban 体系，已由 atom_data 替代，保留未引用） |
| `atom_core` | 日志、基础工具 |
| `atom_math` | 数学工具 |
| `atom_utils` | 通用工具 |
| `atom_layertag` | 图层/标签系统 |
| `atom_renderdoc` | RenderDoc 集成 |
