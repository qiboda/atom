use bevy::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use cel_shader::back_facing::{BackFacingMaterial, BackFacingPlugin};

use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(NoCameraPlayerPlugin);
    app.add_plugins(BackFacingPlugin);

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut back_facing_materials: ResMut<Assets<BackFacingMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::from(Cuboid {
        half_size: Vec3::splat(2.0),
    });

    write_average_normal_to_tangent(&mut mesh);

    // TODO 目前不支持材质排序，等支持后再处理
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: std_materials.add(StandardMaterial {
                base_color: LinearRgba::new(0.8, 0.7, 0.6, 0.7).into(),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        back_facing_materials.add(BackFacingMaterial {
            stroke_color: LinearRgba::RED,
            stroke_width: 0.01,
        }),
    ));

    let mut mesh = Mesh::from(Sphere { radius: 2.0 });
    write_average_normal_to_tangent(&mut mesh);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: std_materials.add(StandardMaterial {
                base_color: LinearRgba::new(0.8, 0.7, 0.6, 0.7).into(),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            ..Default::default()
        },
        back_facing_materials.add(BackFacingMaterial {
            stroke_color: LinearRgba::RED,
            stroke_width: 0.01,
        }),
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        FlyCam,
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(8.0, 8.0, 8.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn write_average_normal_to_tangent(mesh: &mut Mesh) {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");

    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .unwrap()
        .as_float3()
        .expect("`Mesh::ATTRIBUTE_NORMAL` vertex attributes should be of type `float3`");

    let mut vertices = vec![];
    let mut avg_normals: Vec<[f32; 3]> = vec![];
    for (pos_index, vertex) in positions.iter().enumerate() {
        let vertex_index = vertices.iter().position(|x| *x == vertex);
        match vertex_index {
            Some(vertex_index) => {
                avg_normals[vertex_index][0] += normals[pos_index][0];
                avg_normals[vertex_index][1] += normals[pos_index][1];
                avg_normals[vertex_index][2] += normals[pos_index][2];
            }
            None => {
                vertices.push(vertex);
                avg_normals.push(normals[pos_index]);
            }
        }
    }

    for normal in avg_normals.iter_mut() {
        let n = Vec3::from_array(*normal).normalize();
        normal[0] = n.x;
        normal[1] = n.y;
        normal[2] = n.z;
    }

    let mut tangent = vec![];
    for vertex in positions.iter() {
        let vertex_index = vertices.iter().position(|x| *x == vertex).unwrap();
        let normal = avg_normals[vertex_index];
        tangent.push([normal[0], normal[1], normal[2], 0.0]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangent);
}
