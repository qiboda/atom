use bevy::prelude::*;

use crate::terrain::isosurface::{
    meshing::mesh::MeshCache,
    octree::{
        cell::{Cell, CellMeshInfo, CellType},
        octree::{Octree, OctreeCellAddress},
        tables::SubCellIndex,
    },
    sample::surface_sampler::SurfaceSampler,
    surface::shape_surface::ShapeSurface,
    IsosurfaceExtractionState,
};

use strum::IntoEnumIterator;

pub fn tessellation_traversal(
    mut query: Query<(
        &Octree,
        &OctreeCellAddress,
        &mut MeshCache,
        &mut SurfaceSampler,
        &mut IsosurfaceExtractionState,
    )>,
    cells: Query<&Cell>,
    mut cell_mesh_infos: Query<&mut CellMeshInfo>,
    shape_surface: Res<ShapeSurface>,
) {
    for (octree, cell_addresses, mut mesh_cache, mut surface_sampler, mut state) in query.iter_mut()
    {
        if let IsosurfaceExtractionState::Meshing = *state {
            for entity in octree.cells.iter() {
                tessellation_traversal_inner(
                    &mut mesh_cache,
                    *entity,
                    &cells,
                    &mut cell_mesh_infos,
                    cell_addresses,
                    &mut surface_sampler,
                    &shape_surface,
                );
            }
            *state = IsosurfaceExtractionState::Done;
        }
    }
}

fn tessellation_traversal_inner(
    mesh: &mut MeshCache,
    entity: Entity,
    cells: &Query<&Cell>,
    cell_mesh_infos: &mut Query<&mut CellMeshInfo>,
    cell_addresses: &OctreeCellAddress,
    surface_sampler: &mut SurfaceSampler,
    shape_surface: &Res<ShapeSurface>,
) {
    if let Ok(cell) = cells.get(entity) {
        let cell_type = cell.get_cell_type();
        match cell_type {
            CellType::Branch => {
                for subcell_index in SubCellIndex::iter() {
                    let child_address = cell.get_address().get_child_address(subcell_index);
                    let child_cell_entity =
                        cell_addresses.cell_addresses.get(&child_address).unwrap();
                    tessellation_traversal_inner(
                        mesh,
                        *child_cell_entity,
                        cells,
                        cell_mesh_infos,
                        cell_addresses,
                        surface_sampler,
                        shape_surface,
                    );
                }
            }
            CellType::Leaf => {
                let mut cell_mesh_info = cell_mesh_infos.get_mut(entity).unwrap();
                for component in cell_mesh_info.components.iter_mut() {
                    tessellate_component(mesh, surface_sampler, shape_surface, component);
                }
            }
        }
    }
}

fn tessellate_component(
    mesh_cache: &mut MeshCache,
    surface_sampler: &mut SurfaceSampler,
    shape_surface: &Res<ShapeSurface>,
    component: &mut Vec<u32>,
) {
    let num_of_indices = component.len();

    assert!(num_of_indices >= 3);

    if num_of_indices == 3 {
        make_tri(mesh_cache, component);
    } else if num_of_indices > 3 {
        let mut centro_id_pos = Vec3::new(0.0, 0.0, 0.0);
        let mut centro_id_normal = Vec3::new(0.0, 0.0, 0.0);

        for comp in component.iter() {
            centro_id_pos += mesh_cache.positions[*comp as usize];
            centro_id_normal += mesh_cache.normals[*comp as usize];
        }

        let mut med_vertex = Vec3::new(
            centro_id_pos.x / num_of_indices as f32,
            centro_id_pos.y / num_of_indices as f32,
            centro_id_pos.z / num_of_indices as f32,
        );

        if shape_surface.snap_centro_id {
            let med_val = surface_sampler.get_value_from_pos(med_vertex, shape_surface);

            let med_dimension = surface_sampler.voxel_size / 2.0;

            let mut med_gradient = Vec3::new(0.0, 0.0, 0.0);

            find_gradient_with_value(
                surface_sampler,
                shape_surface,
                &mut med_gradient,
                med_dimension,
                med_vertex,
                med_val,
            );

            // 沿着反方向偏移
            med_vertex += -med_gradient * med_val;
        }

        centro_id_normal = centro_id_normal.normalize();

        mesh_cache.positions.push(med_vertex);
        mesh_cache.normals.push(centro_id_normal);

        component.push((mesh_cache.positions.len() - 1) as u32);

        make_tri_fan(mesh_cache, component);
    }
}

/// 3d梯度等价于2d的法线, 3d梯度等于2d方向上最快的变化方向，也就是法线
/// 4d梯度等价于3d的法线, 4d梯度等于3d方向上最快的变化方向，也就是法线
fn find_gradient_with_value(
    surface_sampler: &mut SurfaceSampler,
    shape_surface: &Res<ShapeSurface>,
    normal: &mut Vec3,
    dimensions: Vec3,
    position: Vec3,
    value: f32,
) {
    let dx = surface_sampler
        .get_value_from_pos(position + Vec3::new(dimensions.x, 0.0, 0.0), shape_surface);

    let dy = surface_sampler
        .get_value_from_pos(position + Vec3::new(0.0, dimensions.y, 0.0), shape_surface);

    let dz = surface_sampler
        .get_value_from_pos(position + Vec3::new(0.0, 0.0, dimensions.z), shape_surface);

    *normal = Vec3::new(dx - value, dy - value, dz - value).normalize();
}

fn make_tri(mesh: &mut MeshCache, component: &Vec<u32>) {
    for i in 0..3 {
        mesh.get_indices_mut().push(component[i]);
    }
}

// 扇形三角面
fn make_tri_fan(mesh: &mut MeshCache, component: &Vec<u32>) {
    for i in 0..(component.len() - 2) {
        mesh.get_indices_mut()
            .push(component[component.len() - 1] as u32);
        mesh.get_indices_mut().push(component[i] as u32);
        mesh.get_indices_mut().push(component[i + 1] as u32);
    }

    mesh.get_indices_mut()
        .push(component[component.len() - 1] as u32);
    mesh.get_indices_mut()
        .push(component[component.len() - 2] as u32);
    mesh.get_indices_mut().push(component[0] as u32);
}
