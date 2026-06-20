# TENSIONS.md — Atom 项目摩擦日志

> 发现数据与系统设计之间的不一致、工具链问题、或流程阻碍时记录。不要当场解决——只捕获信号。


## GPU 管线

- **composable module 解析死锁**: shader 在 render app `build()` 里用 `DirectAssetAccessExt::load_asset` 加载 → `AssetEvent<Shader>` 只在 render world 发，永远不被 `PipelineCache::extract_shaders`（只读 main world 事件）捕获 → composable import 注册不发生 → pipeline 永不解锁（`ShaderNotLoaded`/`ShaderImportNotYetAvailable` 交替）。Bevy Game of Life 示例的正确做法：main app `Startup` 里用 `AssetServer::load`，靠 `ExtractResource` 自然同步。涉及所有 `shaders_plugin!` 调用点。
- 主分支 `density_field.wgsl` 有 `h10` undefined bug（WgslParseError），feature 分支修了 `h10` 但引入 noise composable import 链暴露上述 bug。
- WGSL 清理：删掉 `voxel_cross_points.wgsl`/`voxel_utils.wgsl`/`main_mesh_compute_vertices.wgsl` 中不存在的 `VOXEL_MATERIAL_*` 和 `get_voxel_material_type*` import。
- TerrainMaterial 管线不渲染 — 0 vertex shader invocation。已退成 StandardMaterial 白模验证。Biome 颜色不可见。

## 数据对齐

- pack4xU8 biome 编码脆弱 — CPU 端位掩码解包 (`0xFF`, `>>8`, `>>16`, `>>24`)，GPU 布局改则坏。
- mesh_vertex_map 用 u32 索引，单 chunk 上限 4096 顶点。未来 LOD 0 可能超。
- biome 纹理 Luma8 → GPU normalizes to [0,1] → 需 ×255 恢复。漏了就是全零 biome 类型。

## 工具链

- rust-analyzer LSP 初次启动失败（组件未安装）。已装 1.96.0。
- buffer.rs + node.rs Phase 4 编辑损害了 render world 状态一致性 → 已回退 Phase 3 版本。
- WGSL 无断点 — compute shader 调试全靠「改 → 怀疑 → print → 重复」。

## 流程

- 噪声调参被推迟（"先不急"）→ 地形视觉未定。
- 无 per-module 日志等级文件 — 当前全靠 atom_log_plugin 运行时过滤字符串。
- LOD 边界无缝合 — 不同分辨率 chunk 之间有裂缝。

## 已知退化

- Wireframe 已全局关闭，DirectionalLight 需手动加（example 级）。
- 简化 MC 索引（非 256 表）有已知洞（FIXME 标注）。边界 chunk 可能缺面。
