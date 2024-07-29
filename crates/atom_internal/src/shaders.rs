use bevy::{app::Plugin, prelude::*};

#[derive(Resource, Default)]
pub struct AtomInternalShaders {
    pub simplex_shader: Handle<Shader>,
    pub morton_shader: Handle<Shader>,
}

#[derive(Default)]
pub struct AtomShadersPlugin;

impl Plugin for AtomShadersPlugin {
    fn build(&self, app: &mut App) {
        let shaders = AtomInternalShaders {
            simplex_shader: app.world().load_asset("shaders/noise/simplex.wgsl"),
            morton_shader: app.world().load_asset("shaders/utils/morton.wgsl"),
        };

        app.insert_resource(shaders);
    }
}
