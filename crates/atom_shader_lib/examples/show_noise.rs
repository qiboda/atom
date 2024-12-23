use std::path::Path;

use atom_shader_lib::{AtomLimitShadersPlugin, AtomNoiseShadersPlugin, AtomRandomShadersPlugin};
use bevy::{
    asset::{embedded_asset, io::AssetSourceId, AssetPath},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<CustomMaterial>::default())
        .add_plugins(AtomNoiseShadersPlugin)
        .add_plugins(AtomRandomShadersPlugin)
        .add_plugins(AtomLimitShadersPlugin)
        .add_systems(Startup, setup);

    let omit_prefix = "crates/atom_shader_lib/examples";
    embedded_asset!(app, omit_prefix, "show_noise.wgsl");

    app.run();
}

fn setup(
    mut commands: Commands,
    windows: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    commands.spawn(Camera2d);

    let window = windows.single();
    let size = window.resolution.size();

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(materials.add(CustomMaterial {})),
    ));
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        let path = Path::new("show_noise/show_noise.wgsl");
        let source = AssetSourceId::from("embedded");
        let asset_path = AssetPath::from_path(path).with_source(source);
        asset_path.into()
    }
}
