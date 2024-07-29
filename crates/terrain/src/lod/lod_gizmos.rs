use bevy::{math::bounding::BoundingVolume, prelude::*};

use super::lod_octree::TerrainLodOctree;

#[derive(Debug, Default)]
pub struct TerrainLodGizmosPlugin;

impl Plugin for TerrainLodGizmosPlugin {
    fn build(&self, app: &mut App) {
        return;
        app.init_gizmo_group::<WorldCoordinateGizmos>()
            .init_gizmo_group::<VoxelGizmos>()
            .add_systems(Update, update_config)
            .add_systems(Update, draw_world_coordinate_axes)
            .add_systems(Update, draw_lod_octree_voxel);
    }
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct WorldCoordinateGizmos;

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct VoxelGizmos;

// TODO: move to other debug crate
#[allow(dead_code)]
fn draw_world_coordinate_axes(mut world_coordinate_gizmos: Gizmos<WorldCoordinateGizmos>) {
    world_coordinate_gizmos.axes(
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::from_axis_angle(Vec3::X, 0.0),
            scale: Vec3::ONE,
        },
        60.0,
    );
}

// #[allow(dead_code)]
// fn draw_voxel_normal(query: Query<&Octree>, mut octree_node_gizmos: Gizmos<VoxelGizmos>) {
//     for octree in query.iter() {
//         let node_addresses = octree.address_node_map.read().unwrap();
//         if node_addresses.is_empty().not() {
//             debug!("node num: {}", node_addresses.len());
//             for (_, node) in node_addresses.iter() {
//                 if node.node_type == NodeType::Leaf {
//                     let loc = node.vertex_estimate;
//                     let normal = node.normal_estimate;

//                     octree_node_gizmos.arrow(loc, loc + Vec3::from(normal) * 2.0, css::RED);
//                 }
//             }
//         }
//     }
// }

#[allow(dead_code)]
fn draw_lod_octree_voxel(
    terrain_lod_octree: Res<TerrainLodOctree>,
    mut octree_node_gizmos: Gizmos<VoxelGizmos>,
) {
    let mut len = 0;
    for level in terrain_lod_octree.octree_levels.iter() {
        len += level.get_current().len();

        for node in level.get_current().iter() {
            octree_node_gizmos.cuboid(
                Transform {
                    translation: node.aabb.center().into(),
                    rotation: Quat::IDENTITY,
                    scale: (node.aabb.half_size() * 2.0).into(),
                },
                match node.code.level() {
                    0 => LinearRgba::BLACK,
                    1 => LinearRgba::RED,
                    2 => LinearRgba::GREEN,
                    3 => LinearRgba::BLUE,
                    4 => LinearRgba::RED,
                    5 => LinearRgba::GREEN,
                    6 => LinearRgba::BLUE,
                    7 => LinearRgba::RED,
                    8 => LinearRgba::GREEN,
                    _ => LinearRgba::BLUE,
                },
            );
        }
    }
}

fn update_config(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, config, _) in config_store.iter_mut() {
        config.depth_bias = 0.0;
        config.line_width = 200.0;
        config.line_perspective = true;
    }
}

// #[allow(dead_code)]
// fn draw_seam_octree_voxel(
//     query: Query<&Octree, With<TerrainChunkSeamGenerator>>,
//     mut octree_node_gizmos: Gizmos<VoxelGizmos>,
// ) {
//     for octree in query.iter() {
//         let node_addresses = octree.address_node_map.read().unwrap();
//         if node_addresses.is_empty().not() {
//             for (address, node) in node_addresses.iter() {
//                 if node.node_type == NodeType::Leaf {
//                     octree_node_gizmos.cuboid(
//                         Transform {
//                             translation: node.aabb.center().into(),
//                             rotation: Quat::IDENTITY,
//                             scale: (node.aabb.half_size() * 2.0).into(),
//                         },
//                         match address.depth() {
//                             1 => LinearRgba::RED,
//                             2 => LinearRgba::GREEN,
//                             3 => LinearRgba::BLUE,
//                             4 => LinearRgba::RED,
//                             5 => LinearRgba::GREEN,
//                             6 => LinearRgba::BLUE,
//                             7 => LinearRgba::RED,
//                             8 => LinearRgba::GREEN,
//                             9 => LinearRgba::BLUE,
//                             10 => LinearRgba::RED,
//                             11 => LinearRgba::GREEN,
//                             12 => LinearRgba::BLUE,
//                             13 => LinearRgba::RED,
//                             _ => LinearRgba::GREEN,
//                         },
//                     );
//                 }
//             }
//         }
//     }
// }
