//! 线段图元：`LineList` 拓扑的材质、网格构造与插件注册。

/// 线段渲染材质（`LineMaterial`）与 shader 设置。
pub mod material;
/// 线段网格数据（`LineMesh`）及其到 Bevy `Mesh` 的转换。
pub mod mesh;
/// 注册线段 shader 与材质的插件。
pub mod plugin;
