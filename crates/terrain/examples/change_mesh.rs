use bevy::prelude::{Sphere, *};

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

    let sphere = Sphere::new(1.0);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(sphere)),
        material: materials.add(StandardMaterial::from_color(LinearRgba::new(
            0.8, 0.7, 0.6, 1.0,
        ))),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        visibility: Visibility::Hidden,
        ..Default::default()
    });
}

fn replace_mesh(
    key: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Visibility, &Handle<Mesh>)>,
) {
    if key.just_pressed(KeyCode::KeyM) {
        for (mut vis, _mesh) in query.iter_mut() {
            match *vis {
                Visibility::Inherited => {}
                Visibility::Hidden => *vis = Visibility::Visible,
                Visibility::Visible => *vis = Visibility::Hidden,
            }
        }
    }
}
