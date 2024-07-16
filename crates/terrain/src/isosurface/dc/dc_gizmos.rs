use std::ops::Not;

use bevy::{color::palettes::css, math::bounding::BoundingVolume, prelude::*};

use crate::isosurface::dc::octree::cell::CellType;

use super::octree::Octree;

#[derive(Debug, Default)]
pub struct DcGizmosPlugin;

impl Plugin for DcGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<WorldCoordinateGizmos>()
            .init_gizmo_group::<VoxelGizmos>()
            .add_systems(Update, draw_world_coordinate_axes)
            // .add_systems(Update, draw_octree_voxel)
            .add_systems(Update, draw_voxel_normal);
    }
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct WorldCoordinateGizmos;

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct VoxelGizmos;

fn draw_world_coordinate_axes(mut world_coordinate_gizmos: Gizmos<WorldCoordinateGizmos>) {
    world_coordinate_gizmos.axes(
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::from_axis_angle(Vec3::X, 0.0),
            scale: Vec3::ONE,
        },
        3.0,
    );
}

fn draw_voxel_normal(query: Query<&Octree>, mut octree_cell_gizmos: Gizmos<VoxelGizmos>) {
    for octree in query.iter() {
        let cell_addresses = octree.address_cell_map.read().unwrap();
        if cell_addresses.is_empty().not() {
            debug!("cell num: {}", cell_addresses.len());
            for (_, cell) in cell_addresses.iter() {
                if cell.cell_type == CellType::Leaf {
                    let loc = cell.vertex_estimate;
                    let normal = cell.normal_estimate;

                    octree_cell_gizmos.arrow(loc, loc + Vec3::from(normal) * 2.0, css::RED);
                }
            }
        }
    }
}

fn draw_octree_voxel(query: Query<&Octree>, mut octree_cell_gizmos: Gizmos<VoxelGizmos>) {
    for octree in query.iter() {
        let cell_addresses = octree.address_cell_map.read().unwrap();
        if cell_addresses.is_empty().not() {
            debug!("cell num: {}", cell_addresses.len());
            for (address, cell) in cell_addresses.iter() {
                debug!("pos: {}", cell.vertex_estimate);
                octree_cell_gizmos.cuboid(
                    Transform {
                        translation: cell.aabb.center().into(),
                        rotation: Quat::IDENTITY,
                        scale: (cell.aabb.half_size() * 2.0).into(),
                    },
                    match address.depth() {
                        1 => css::RED,
                        2 => css::GREEN,
                        3 => css::BLUE,
                        4 => css::YELLOW,
                        5 => css::ORANGE,
                        6 => css::MAGENTA,
                        7 => css::WHITE,
                        _ => css::BLACK,
                    },
                );
            }
        }
    }
}
