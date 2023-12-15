use leafwing_input_manager::{prelude::*, InputManagerBundle};

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::{command, Parser};

use bevy::prelude::*;
use bevy::render::render_resource::TextureViewDimension::Cube;
use bevy::render::view::NoFrustumCulling;
use bevy_debug_grid::DebugGridPlugin;

use crate::player::geometry_data::process_player_order;
use camera::SmoothCameraPlugin;
use player::{geometry_data::GeometryData, PlayerOrders};
use shapes::{
    lines::material::LineMaterial,
    points::{material::PointsMaterial, mesh::PointsMesh},
    triangles::{material::TriangleMaterial, mesh::TrianglesMesh},
};
use terrain_player_client::OrderType;

use crate::shapes::triangles::plugin::TrianglesPlugin;
use crate::shapes::{lines::plugin::LinesPlugin, points::plugin::PointsPlugin};

mod camera;
mod player;
mod shapes;

#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    filename: PathBuf,
}

fn main() {
    let args = Args::parse();
    info!("filename: {:?}", args.filename);

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<InputAction>::default())
        .add_plugins(PointsPlugin)
        .add_plugins(LinesPlugin)
        .add_plugins(TrianglesPlugin)
        .add_plugins(SmoothCameraPlugin)
        .add_plugins(DebugGridPlugin::with_floor_grid())
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
        .insert_resource(args)
        .insert_resource(PlayerOrders::default())
        .insert_resource(GeometryData::default())
        .add_systems(
            Startup,
            (
                startup,
                process_player_order,
                spawn_point_and_line,
                spawn_input_action,
            )
                .chain(),
        )
        .add_systems(Update, (next_order, pre_order));

    app.run();
}

// 单一光标。
// 启动和清除不同的线程。
fn startup(args: Res<Args>, mut player_order: ResMut<PlayerOrders>) {
    if let Ok(file) = File::open(args.filename.clone()) {
        let reader = BufReader::new(file);
        let mut line_num = 0;
        for line in reader.lines().flatten() {
            match serde_json::from_str(line.as_str()) {
                Ok(order) => {
                    player_order.push_order(order);
                    line_num += 1;
                }
                Err(err) => {
                    error!("parse line error: {}, err: {}", line, err);
                }
            }
        }
        info!("file line num: {}", line_num);
    } else {
        error!("parse file name can not open: {:?}", args.filename);
    }
}

#[derive(Component, Debug)]
struct Point;

#[derive(Component, Debug)]
struct Line;

#[derive(Component, Debug)]
struct Triangle;

fn spawn_input_action(mut commands: Commands) {
    commands.spawn(InputManagerBundle::<InputAction> {
        action_state: ActionState::<InputAction>::default(),
        input_map: InputMap::new([
            (KeyCode::N, InputAction::NextOrder),
            (KeyCode::P, InputAction::PreOrder),
        ]),
    });
}

fn spawn_point_and_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut geometry_data: Res<GeometryData>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut points_materials: ResMut<Assets<PointsMaterial>>,
    mut triangle_materials: ResMut<Assets<TriangleMaterial>>,
) {
    let vertices = geometry_data.vertices.clone();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube::new(1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        NoFrustumCulling,
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(PointsMesh::default())),
            material: points_materials.add(PointsMaterial::default()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        Point,
        NoFrustumCulling,
    ));

    // commands.spawn((
    //     MaterialMeshBundle {
    //         mesh: meshes.add(Mesh::from(LineMesh::default())),
    //         material: line_materials.add(LineMaterial::default()),
    //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    //         ..Default::default()
    //     },
    //     Line,
    //     NoFrustumCulling,
    // ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(TrianglesMesh::build_mesh(
                Some(vec![[0.0; 3], [0.0; 3], [0.0; 3]]),
                Some(vec![0, 1, 2]),
            ))),
            material: triangle_materials.add(TriangleMaterial::default()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        Triangle,
        NoFrustumCulling,
    ));
}

#[derive(Actionlike, Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
enum InputAction {
    NextOrder,
    PreOrder,
}

fn next_order(
    input_query: Query<&ActionState<InputAction>>,
    mut player_order: ResMut<PlayerOrders>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut geometry_data: Res<GeometryData>,
    mut point_query: Query<&Handle<Mesh>, With<Point>>,
    mut line_query: Query<&Handle<Mesh>, With<Line>>,
    mut triangle_query: Query<&Handle<Mesh>, With<Triangle>>,
) {
    let action_state = input_query.single();

    let mut count = 0;
    loop {
        // if action_state.pressed(InputAction::NextOrder) {
        //     info!("next order");
        if let Some(order) = player_order.next() {
            match order.fields.order_type {
                OrderType::Vertex(data) => {
                    let mesh = point_query.single_mut();
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        PointsMesh::add_point(mesh, data.location);
                    }
                }
                OrderType::Edge(data) => {}
                OrderType::Triangle(_data) => {
                    let mesh = triangle_query.single_mut();
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        if let Some(len) = TrianglesMesh::get_indices_len(mesh) {
                            let mut indices = [0; 3];
                            for i in 0..3 {
                                if let Some(index) = geometry_data.triangle_indices.get(len + i) {
                                    indices[i] = *index;
                                }
                            }
                            TrianglesMesh::add_all_vertices(mesh, geometry_data.vertices.clone());
                            TrianglesMesh::add_triangle_indices(mesh, indices);
                        }
                    }
                }
            }
        }
        // }

        count += 1;
        if count > 100 {
            break;
        }
    }
}

fn pre_order(
    input_query: Query<&ActionState<InputAction>>,
    mut player_order: ResMut<PlayerOrders>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut point_query: Query<&Handle<Mesh>, With<Point>>,
    mut line_query: Query<&Handle<Mesh>, With<Line>>,
    mut triangle_materials: ResMut<Assets<TriangleMaterial>>,
) {
    let action_state = input_query.single();

    if action_state.pressed(InputAction::PreOrder) {
        // info!("pre order");
        if let Some(order) = player_order.pre() {
            match order.fields.order_type {
                OrderType::Vertex(data) => {
                    let mesh = point_query.single_mut();
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        if let Some(index) = PointsMesh::get_last_index(mesh) {
                            PointsMesh::remove_point_at_index(mesh, index);
                        }
                    }
                }
                OrderType::Edge(data) => {}
                OrderType::Triangle(data) => {
                    let mesh = point_query.single_mut();
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        if let Some(len) = TrianglesMesh::get_indices_len(mesh) {
                            TrianglesMesh::remove_last_triangle_indices(mesh);
                        }
                    }
                }
            }
        }
    }
}
