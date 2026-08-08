#![deny(missing_docs)]

//! Shader 工具库：为引擎提供常用 GPU shader 的加载插件（数学、噪声、随机、CSG、Morton 编码等），
//! 以及点/线/三角形等基础图元的材质与网格构造工具。

use bevy::app::{PluginGroup, PluginGroupBuilder};
use shader_plugin_macro::atom_shaders_plugin;

mod gpu_test;
/// shader 插件声明宏（`shaders_plugin!` 等）的定义与重导出。
pub mod shader_plugin_macro;
/// 基础图元（线 / 点 / 三角形）的材质、网格与插件。
pub mod shapes;

/// 一次性注册全部 Atom shader 插件的插件组。
///
/// 包含 [`AtomLimitShadersPlugin`]、[`AtomMathShadersPlugin`]、
/// [`AtomNoiseShadersPlugin`]、[`AtomMortonShadersPlugin`]、[`AtomRandomShadersPlugin`] 与
/// [`AtomCSGShadersPlugin`] 六个插件。
#[derive(Default)]
pub struct AtomShaderLibPluginGroups;

impl PluginGroup for AtomShaderLibPluginGroups {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<AtomShaderLibPluginGroups>();
        group
            .add(AtomLimitShadersPlugin)
            .add(AtomMathShadersPlugin)
            .add(AtomNoiseShadersPlugin)
            .add(AtomMortonShadersPlugin)
            .add(AtomRandomShadersPlugin)
            .add(AtomCSGShadersPlugin)
    }
}

atom_shaders_plugin!(
    CSG,
    (
        csg_operate_shader -> "shaders/csg/csg_operate.wgsl",
        csg_shapes_shader -> "shaders/csg/csg_shape.wgsl"
    )
);

atom_shaders_plugin!(Limit, (numeric_shader -> "shaders/limit/numeric.wgsl"));

atom_shaders_plugin!(
    Math,
    (
        const_shader -> "shaders/math/const.wgsl",
        map_shader -> "shaders/math/map.wgsl",
        pack_shader -> "shaders/math/pack.wgsl"
    )
);

atom_shaders_plugin!(
    Morton,
    (morton_shader -> "shaders/utils/morton.wgsl")
);

atom_shaders_plugin!(
    Noise,
    (
        open_simplex_shader -> "shaders/noise/core/open_simplex.wgsl",
        open_simplex_seed_shader -> "shaders/noise/core/open_simplex_seed.wgsl",
        fbm_shader -> "shaders/noise/core/fbm.wgsl",
        ridged_shader -> "shaders/noise/core/ridged.wgsl"
    )
);

atom_shaders_plugin!(
    Random,
    (
        xorshift_32_shader ->  "shaders/random/xorshift_32.wgsl",
        xorshift_128_shader -> "shaders/random/xorshift_128.wgsl",
        taus_lcg_shader -> "shaders/random/taus_lcg.wgsl"
    )
);
