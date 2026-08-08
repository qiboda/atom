#![deny(missing_docs)]
//! QEF（二次误差函数）最小化求解器，算法实现基于论文：
//! *Computer Graphics Forum - 2020 - Trettner - Fast and Robust QEF Minimization using Probabilistic Quadrics*
//! 提供 CPU 端的 Quadric 数学实现（`quadric`/`math` 模块）与 GPU shader 句柄资源（`AtomQuadricPlugin`）。

/// QEF 求解的数学辅助：外积、矩阵迹、协方差统计与概率 quadric 构造所需的矩阵运算。
pub mod math;
/// 二次误差函数（Quadric）的核心数据结构与构造/求解方法。
pub mod quadric;

use bevy::{
    app::{App, Plugin},
    asset::{DirectAssetAccessExt, Handle},
    prelude::{Resource, Shader},
};

/// Bevy 插件：加载 QEF 求解器所需的 WGSL shader 资源（`QuadricShaders`）。
#[derive(Default)]
pub struct AtomQuadricPlugin;

/// QEF 求解器使用的 shader 句柄集合，作为 Resource 插入。
#[derive(Debug, Resource)]
pub struct QuadricShaders {
    /// quadric 计算的 WGSL shader 句柄。
    pub quadric_shader: Handle<Shader>,
    /// 数学工具函数的 WGSL shader 句柄。
    pub math_shader: Handle<Shader>,
}

impl Plugin for AtomQuadricPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world();
        app.insert_resource(QuadricShaders {
            quadric_shader: world.load_asset("shaders/quadric/quadric.wgsl"),
            math_shader: world.load_asset("shaders/quadric/math.wgsl"),
        });
    }
}
