use bevy::{app::Plugin, prelude::*};

#[derive(Resource, Default)]
pub struct AtomMortonShaders {
    pub morton_shader: Handle<Shader>,
}

#[derive(Default)]
pub struct AtomMortonShaderPlugin;

impl Plugin for AtomMortonShaderPlugin {
    fn build(&self, app: &mut App) {
        let shaders = AtomMortonShaders {
            morton_shader: app.world().load_asset("shaders/utils/morton.wgsl"),
        };

        app.insert_resource(shaders);
    }
}
