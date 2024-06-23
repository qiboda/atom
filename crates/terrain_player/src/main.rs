// // use leafwing_input_manager::{prelude::*, InputManagerBundle};

// use std::{
//     fs::File,
//     io::{BufRead, BufReader},
//     path::PathBuf,
// };

// use clap::{command, Parser};

// use bevy::prelude::*;
// use bevy::render::view::NoFrustumCulling;
// // use bevy_debug_grid::DebugGridPlugin;

// // use crate::player::{
// //     geometry_data::process_player_order,
// //     {next_order, pre_order},
// // };
// // use crate::player::{Player, PlayerFilter};
// // use camera::SmoothCameraPlugin;
// // use player::geometry_data::AllGeometryData;
// // use player::order::Orders;
// use shapes::{
//     lines::material::LineMaterial,
//     points::{material::PointsMaterial, mesh::PointsMesh},
//     triangles::{material::TriangleMaterial, mesh::TrianglesMesh},
// };
// use terrain_player_client::{order::Order, trace::terrain_layer};

// use crate::shapes::triangles::plugin::TrianglesPlugin;
// use crate::shapes::{lines::plugin::LinesPlugin, points::plugin::PointsPlugin};

// mod camera;
// mod player;
// mod shapes;

// #[derive(Parser, Debug, Resource)]
// #[command(author, version, about, long_about=None)]
// struct Args {
//     #[arg(short, long)]
//     filename: PathBuf,
// }

// fn main() {
//     let args = Args::parse();
//     info!("filename: {:?}", args.filename);

//     let mut app = App::new();

//     app.add_plugins((
//         DefaultPlugins
//         .set(bevy::log::LogPlugin {
//             custom_layer: terrain_layer,

//             ..default()
//         }))
//     )
//     // .add_plugins(InputManagerPlugin::<InputAction>::default())
//     .add_plugins(PointsPlugin)
//     .add_plugins(LinesPlugin)
//     .add_plugins(TrianglesPlugin)
//     .add_plugins(SmoothCameraPlugin)
//     // .add_plugins(DebugGridPlugin::with_floor_grid())
//     .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
//     .insert_resource(args)
//     .insert_resource(Orders::default())
//     .insert_resource(Player::default())
//     .insert_resource(PlayerFilter::default())
//     .insert_resource(AllGeometryData::default())
//     .add_systems(
//         Startup,
//         (
//             startup,
//             process_player_order,
//             spawn_point_line_triangle,
//             spawn_input_action,
//         )
//             .chain(),
//     )
//     // .add_systems(Update, (next_order, pre_order));

//     app.run();
// }

// // 单一光标。
// // 启动和清除不同的线程。
// // fn startup(args: Res<Args>, mut player_order: ResMut<Orders>) {
// //     if let Ok(file) = File::open(args.filename.clone()) {
// //         let reader = BufReader::new(file);
// //         let mut line_num = 0;
// //         for line in reader.lines().map_while(Result::ok) {
// //             match serde_json::from_str::<Order>(line.as_str()) {
// //                 Ok(order) => {
// //                     player_order.push_order(order);
// //                     line_num += 1;
// //                 }
// //                 Err(err) => {
// //                     error!("parse line error: {}, err: {}", line, err);
// //                 }
// //             }
// //         }
// //         info!("file line num: {}", line_num);
// //     } else {
// //         error!("parse file name can not open: {:?}", args.filename);
// //     }
// // }

// #[derive(Component, Debug, Reflect)]
// pub struct Point;

// #[derive(Component, Debug, Reflect)]
// pub struct Line;

// #[derive(Component, Debug, Reflect)]
// pub struct Triangle;

// fn spawn_input_action(mut commands: Commands) {
//     // commands.spawn(InputManagerBundle::<InputAction> {
//     //     action_state: ActionState::<InputAction>::default(),
//     //     input_map: InputMap::new([
//     //         (InputAction::PreOrder, KeyCode::KeyJ),
//     //         (InputAction::NextOrder, KeyCode::KeyK),
//     //         (InputAction::PreHundredOrder, KeyCode::KeyP),
//     //         (InputAction::NextHundredOrder, KeyCode::KeyN),
//     //     ]),
//     // });
// }

// fn spawn_point_line_triangle(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     all_geometry_data: Res<AllGeometryData>,
//     _line_materials: ResMut<Assets<LineMaterial>>,
//     mut points_materials: ResMut<Assets<PointsMaterial>>,
//     mut triangle_materials: ResMut<Assets<TriangleMaterial>>,
// ) {
//     commands.spawn((
//         PbrBundle {
//             mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//             ..Default::default()
//         },
//         NoFrustumCulling,
//     ));

//     for (terrain_chunk_coord, _) in all_geometry_data.geometry_data_map.iter() {
//         commands.spawn((
//             MaterialMeshBundle {
//                 mesh: meshes.add(Mesh::from(PointsMesh::default())),
//                 material: points_materials.add(PointsMaterial::default()),
//                 transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//                 ..Default::default()
//             },
//             Point,
//             NoFrustumCulling,
//             *terrain_chunk_coord,
//         ));

//         // commands.spawn((
//         //     MaterialMeshBundle {
//         //         mesh: meshes.add(Mesh::from(LineMesh::default())),
//         //         material: line_materials.add(LineMaterial::default()),
//         //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//         //         ..Default::default()
//         //     },
//         //     Line,
//         //     NoFrustumCulling,
//         // ));

//         commands.spawn((
//             MaterialMeshBundle {
//                 mesh: meshes.add(TrianglesMesh::build_mesh(Some(vec![]), Some(vec![]))),
//                 material: triangle_materials.add(TriangleMaterial::default()),
//                 transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//                 ..Default::default()
//             },
//             Triangle,
//             NoFrustumCulling,
//             *terrain_chunk_coord,
//         ));
//     }
// }

// // #[derive(Actionlike, Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
// // pub enum InputAction {
// //     NextOrder,
// //     PreOrder,
// //     NextHundredOrder,
// //     PreHundredOrder,
// // }

fn main() {}
