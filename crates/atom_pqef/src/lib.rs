//! 论文是下面这个
//! Computer Graphics Forum - 2020 - Trettner - Fast and Robust QEF Minimization using Probabilistic Quadrics
pub mod math;
pub mod quadric;

use bevy::{
    app::{App, Plugin},
    asset::{DirectAssetAccessExt, Handle},
    prelude::{Resource, Shader},
};

#[derive(Default)]
pub struct QuadricPlugin;

#[derive(Debug, Resource)]
pub struct QuadricShaders {
    pub quadric_shader: Handle<Shader>,
    pub math_shader: Handle<Shader>,
}

impl Plugin for QuadricPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world();
        app.insert_resource(QuadricShaders {
            quadric_shader: world.load_asset("shaders/quadric/quadric.wgsl"),
            math_shader: world.load_asset("shaders/quadric/math.wgsl"),
        });
    }
}
