spec: project
name: "Atom 程序化地形引擎"
tags: [terrain, bevy, gpu, rust]
---

## 意图

构建基于 Bevy 引擎的程序化体素地形系统。GPU Dual Contouring 管线生成平滑 isosurface 网格，CPU Voronoi 图驱动 biome 分布，PBR 材质系统渲染不同生态。

## 约束

- Rust Edition 2024, Bevy 0.18, wgpu 27.0
- 不重新设计现有架构（分层、channel 通信、buffer 抽象）
- 不引入新依赖（详见 SOUL.md Dependencies 规则）

## 排除范围

- 寻路 (nav mesh)
- 怪物 AI 生成规则
- 玩家编辑模式的 CSG
- 网络同步

## 验收标准

场景: workspace 编译通过
  测试: cargo_check_workspace
  假设 Rust 工具链已安装
  当 执行 cargo check --workspace
  那么 零错误
