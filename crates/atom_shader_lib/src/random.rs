use bevy::asset::Handle;
use bevy::{app::Plugin, prelude::*};

const XORSHIFT_32_SHADER_PATH: &str = "shaders/random/xorshift_32.wgsl";
const XORSHIFT_128_SHADER_PATH: &str = "shaders/random/xorshift_128.wgsl";
const TAUS_LCG_SHADER_PATH: &str = "shaders/random/taus_lcg.wgsl";

#[derive(Debug, Default, Resource)]
pub struct AtomRandomShaders {
    pub xorshift_32_shader: Handle<Shader>,
    pub xorshift_128_shader: Handle<Shader>,
    pub taus_lcg_shader: Handle<Shader>,
}

#[derive(Debug, Default)]
pub struct AtomRandomShaderPlugin;

impl Plugin for AtomRandomShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let world = app.world();
        app.insert_resource(AtomRandomShaders {
            xorshift_128_shader: world.load_asset(XORSHIFT_128_SHADER_PATH),
            xorshift_32_shader: world.load_asset(XORSHIFT_32_SHADER_PATH),
            taus_lcg_shader: world.load_asset(TAUS_LCG_SHADER_PATH),
        });
    }
}
