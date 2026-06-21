# Capability Map — Atom 项目


## ✅ 现在能做的
- Rust 代码编写与重构（当前仅 atom_terrain，其余 11 个 crate 待迁入 Bevy 0.19）
- WGSL compute/render shader 编写
- Bevy ECS 系统、状态机、SystemParam 实现
- GPU buffer 管理（直接 wgpu，atom_render 待迁回）
- 数据表配置（atom_datatables / Luban 生成代码 — 待迁回）
- Agent sidecar（TypeScript/BRP）：远程查询 ECS、spawn 实体
- 项目构建、Clippy、测试
- 文件结构导航和代码搜索

## ⚠️ 已知缺口（可解决）

| 缺口 | 能解决？ | 怎么做 |
|------|---------|--------|
| 视觉验证（shader 输出是否正确） | Yes | 用户运行 `cargo run -p atom_terrain --example chunk_loader` |
| RenderDoc 分析 | Yes | 用户 F12 捕获，分析 `saved/renderdoc/` |
| Luban 代码生成 | Yes | 用户运行 `gen_bin.bat`（需 .NET 环境） |
| Python pre-commit | Yes | 安装 uv 后 `cd tools/pythons && uv run pre-commit run` |

## 🔴 始终需要人类

- Shader 视觉效果判断（渲染质量、光照参数调优）
- 地形参数调优（voxel size、chunk 范围、密度场函数选择）
- 游戏设计决策（biome 类型、怪物、技能参数）
- 新 crate 引入决策
- GPU 性能瓶颈判断和优化方向
