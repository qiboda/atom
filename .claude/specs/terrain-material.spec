spec: task
name: "地形渲染材质"
inherits: project
tags: [terrain, material, pbr]
---

## 意图

MVP 阶段使用 Bevy 默认 StandardMaterial 渲染地形 mesh（单色绿色，perceptual_roughness 0.9）。后续 phase 替换为自定义 TerrainMaterial，支持 biome 驱动的 PBR 参数。

## 已定决策

- **MVP**: `StandardMaterial { base_color: Color::srgb(0.4, 0.6, 0.3), perceptual_roughness: 0.9, ..default() }`
- **后续**: 实现 `TerrainMaterial` 实现 `Material` trait，在 fragment shader 中根据 biome 类型选择不同的 albedo/roughness/metallic
- **渲染管线**: 使用 Bevy `MaterialPlugin`（后续 `MaterialPlugin::<TerrainMaterial>`），不自定义渲染管线
- **Biome 推迟**: biome 类型和 biome 颜色数组在 biome phase 实现时加入

## 边界

### 禁止更改

- `assets/shaders/terrain/compute/` — compute shader 不受材质影响
- `crates/atom_terrain/src/compute/` — compute pipeline

### 允许更改

- `crates/atom_terrain/src/mesh.rs` 中的 `StandardMaterial` 初始化参数
- 后续: 新建 `assets/shaders/terrain/render/` 下的 vertex/fragment shader
- 后续: 新建 `crates/atom_terrain/src/material/` 下的 TerrainMaterial 实现

## 完成条件

场景: chunk mesh 以绿色渲染
  测试: chunk_renders_green
  假设 chunk 已完成 mesh 生成
  当 渲染帧
  那么 mesh 可见且为绿色

场景: 光照响应正确
  测试: chunk_responds_to_light
  假设 DirectionalLight 存在
  当 摄像机旋转
  那么 地形表面明暗随光照方向变化
