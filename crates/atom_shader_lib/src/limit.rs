use bevy::asset::Handle;
use bevy::{app::Plugin, prelude::*};

const NUMERIC_SHADER_PATH: &str = "shaders/limit/numeric.wgsl";

#[derive(Debug, Default, Resource)]
pub struct AtomLimitShaders {
    pub numeric_shader: Handle<Shader>,
}

#[derive(Debug, Default)]
pub struct AtomLimitShaderPlugin;

impl Plugin for AtomLimitShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let world = app.world();
        app.insert_resource(AtomLimitShaders {
            numeric_shader: world.load_asset(NUMERIC_SHADER_PATH),
        });
    }
}
