use bevy::{
    prelude::{Sphere, *},
    render::render_asset::RenderAssetUsages,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, replace_mesh)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });

    let cuboid = Cuboid::new(1.0, 1.0, 1.0);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(cuboid)),
        material: materials.add(StandardMaterial::from_color(LinearRgba::new(
            0.8, 0.7, 0.6, 1.0,
        ))),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });
}

fn replace_mesh(
    key: Res<ButtonInput<KeyCode>>,
    query: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut is_sphere: Local<bool>,
) {
    if key.just_pressed(KeyCode::KeyM) {
        for mesh in query.iter() {
            *is_sphere = !*is_sphere;
            if *is_sphere {
                let mesh = meshes.get_mut(mesh).unwrap();
                let sphere = Sphere::new(1.0).mesh();
                *mesh = Mesh::from(sphere);
                mesh.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
            } else {
                let mesh = meshes.get_mut(mesh).unwrap();
                let cylinder = Cylinder::new(1.0, 2.0).mesh();
                *mesh = Mesh::from(cylinder);
                mesh.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
            }
        }
    }
}
