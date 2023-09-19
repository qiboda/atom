use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use crate::terrain::isosurface::{
    cms::build::{address::VoxelAddress, cell::CellType, octree::Octree},
    mesh::mesh_cache::MeshCache,
};

impl Octree {
    pub fn tessellation_traversal(&mut self, mesh_cache: Arc<RwLock<MeshCache>>) {
        info_span!("tessellation_traversal");
        info!("tessellation_traversal");
        for address in self.leaf_cells.clone().iter() {
            self.tessellation_traversal_inner(&mesh_cache, *address);
        }
    }

    fn tessellation_traversal_inner(
        &mut self,
        mesh: &Arc<RwLock<MeshCache>>,
        cell_address: VoxelAddress,
    ) {
        // debug!("tessellation_inner: {:?}", entity);
        if let Some(cell) = self.cell_addresses.get_mut(&cell_address) {
            let cell_type = cell.get_cell_type();
            debug_assert!(cell_type == &CellType::Leaf);
            if cell_type == &CellType::Leaf {
                info!("cell.components {:?}", cell.components);
                cell.components.as_mut().map(|components| {
                    for component in components.iter_mut() {
                        Self::tessellate_component(mesh, component);
                        info!("tessellate_component");
                    }
                    components
                });
            }
        }
    }

    fn tessellate_component(mesh_cache: &Arc<RwLock<MeshCache>>, component: &mut Vec<u32>) {
        let mut mesh_cache = mesh_cache.write().unwrap();

        debug!("tessellate_component");
        let num_of_indices = component.len();

        debug_assert!(num_of_indices >= 3);

        match num_of_indices {
            3 => {
                Self::make_tri(&mut mesh_cache, component);
            }
            num_of_indices if num_of_indices > 3 => {
                let mut centro_id_pos = Vec3::new(0.0, 0.0, 0.0);
                let mut centro_id_normal = Vec3::new(0.0, 0.0, 0.0);

                for comp in component.iter() {
                    centro_id_pos += mesh_cache.positions[*comp as usize];
                    centro_id_normal += mesh_cache.normals[*comp as usize];
                }

                let med_vertex = Vec3::new(
                    centro_id_pos.x / num_of_indices as f32,
                    centro_id_pos.y / num_of_indices as f32,
                    centro_id_pos.z / num_of_indices as f32,
                );

                centro_id_normal = centro_id_normal.normalize();

                mesh_cache.positions.push(med_vertex);
                mesh_cache.normals.push(centro_id_normal);

                debug_assert!(!mesh_cache.positions.is_empty());
                component.push((mesh_cache.positions.len() - 1) as u32);

                Self::make_tri_fan(&mut mesh_cache, component);
            }
            _ => {}
        }
    }

    fn make_tri(mesh: &mut MeshCache, component: &Vec<u32>) {
        debug!("make_tri:{:?}", component);
        // 逆时针
        mesh.get_indices_mut().push(component[0]);
        mesh.get_indices_mut().push(component[2]);
        mesh.get_indices_mut().push(component[1]);
    }

    // 扇形三角面
    fn make_tri_fan(mesh: &mut MeshCache, component: &Vec<u32>) {
        debug!("make_tri_fan: {:?}", component);
        // 逆时针
        for i in 0..(component.len() - 2) {
            mesh.get_indices_mut().push(component[component.len() - 1]);
            mesh.get_indices_mut().push(component[i + 1]);
            mesh.get_indices_mut().push(component[i]);
        }

        // 逆时针
        mesh.get_indices_mut().push(component[component.len() - 2]);
        mesh.get_indices_mut().push(component[component.len() - 1]);
        mesh.get_indices_mut().push(component[0]);
    }
}
