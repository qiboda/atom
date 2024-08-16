use bevy::asset::Handle;
use bevy::{app::Plugin, prelude::*};

const CONST_SHADER_PATH: &str = "shaders/math/const.wgsl";

#[derive(Debug, Default, Resource)]
pub struct AtomMathShaders {
    pub const_shader: Handle<Shader>,
}

#[derive(Debug, Default)]
pub struct AtomMathShaderPlugin;

impl Plugin for AtomMathShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let world = app.world();
        app.insert_resource(AtomMathShaders {
            const_shader: world.load_asset(CONST_SHADER_PATH),
        });
    }
}
