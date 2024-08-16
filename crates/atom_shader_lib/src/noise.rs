use bevy::asset::Handle;
use bevy::{app::Plugin, prelude::*};

const OPEN_SIMPLEX_SHADER_PATH: &str = "shaders/noise/core/open_simplex.wgsl";
const OPEN_SIMPLEX_SEED_SHADER_PATH: &str = "shaders/noise/core/open_simplex_seed.wgsl";
const FBM_SHADER_PATH: &str = "shaders/noise/core/fbm.wgsl";
const RIDGED_SHADER_PATH: &str = "shaders/noise/core/ridged.wgsl";

#[derive(Debug, Default, Resource)]
pub struct AtomNoiseShaders {
    pub open_simplex_shader: Handle<Shader>,
    pub open_simplex_seed_shader: Handle<Shader>,
    pub fbm_shader: Handle<Shader>,
    pub ridged_shader: Handle<Shader>,
}

#[derive(Debug, Default)]
pub struct AtomNoiseShaderPlugin;

impl Plugin for AtomNoiseShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let world = app.world();
        app.insert_resource(AtomNoiseShaders {
            open_simplex_shader: world.load_asset(OPEN_SIMPLEX_SHADER_PATH),
            open_simplex_seed_shader: world.load_asset(OPEN_SIMPLEX_SEED_SHADER_PATH),
            fbm_shader: world.load_asset(FBM_SHADER_PATH),
            ridged_shader: world.load_asset(RIDGED_SHADER_PATH),
        });
    }
}
