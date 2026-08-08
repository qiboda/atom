//! 点精灵图元：以四边形面片 + 点 shader 实现的点渲染，支持透视缩放与圆形裁剪。

/// 点精灵渲染材质（`PointsMaterial`）与 shader 设置。
pub mod material;
/// 点精灵网格数据（`PointsMesh`）及其到 Bevy `Mesh` 的转换与动态增删点。
pub mod mesh;
/// 注册点精灵 shader 与材质的插件。
pub mod plugin;
