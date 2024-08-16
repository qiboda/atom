use std::path::Path;

use atom_shader_lib::{limit::AtomLimitShaderPlugin, noise::AtomNoiseShaderPlugin, random::AtomRandomShaderPlugin};
use bevy::{
    asset::{embedded_asset, io::AssetSourceId, AssetPath},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<CustomMaterial>::default())
        .add_plugins(AtomNoiseShaderPlugin)
        .add_plugins(AtomRandomShaderPlugin)
        .add_plugins(AtomLimitShaderPlugin)
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
    commands.spawn(Camera2dBundle::default());

    let window = windows.single();
    let size = window.resolution.size();

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::new(size.x, size.y)).into(),
        transform: Transform::default(),
        material: materials.add(CustomMaterial {}),
        ..default()
    });
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
