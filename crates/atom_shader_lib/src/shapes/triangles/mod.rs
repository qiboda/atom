//! 三角形图元：`TriangleList` 拓扑的材质、网格构造与插件注册。

/// 三角形渲染材质（`TriangleMaterial`）与 shader 设置。
pub mod material;
/// 三角形网格构造（`TrianglesMesh`）及索引/顶点的增量修改。
pub mod mesh;
/// 注册三角形 shader 与材质的插件。
pub mod plugin;
