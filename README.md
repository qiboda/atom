# Atom Terrain Engine

基于 Bevy 的体素平滑地形引擎。

## 安装

```toml
[dependencies]
atom_terrain = { git = "https://...", branch = "main" }
```

## 使用

```bash
cargo run -p atom_terrain --example chunk_loader
```

## Crate

| Crate | 用途 |
|---|---|
| `atom_terrain` | 核心地形系统 |
| `atom_pqef` | QEF 求解器 |
| `atom_render` | GPU Buffer |
| `atom_shader_lib` | Shader 工具 |
| `atom_cel_shader` | 赛璐璐渲染 |
| `atom_ability` | 技能系统 |
| `atom_datatables` | 数据表加载 |
