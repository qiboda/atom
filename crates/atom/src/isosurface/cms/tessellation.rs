// pub fn tessellation_traversal(
//     cells: Query<&Cell>,
//     query: Query<&Octree, &OctreeCellAddress>,
//     mesh: Query<&mut Mesh>,
//     cms_query: Query<&CMSMeshInfo>,
//     shape_surface: Res<ShapeSurface>,
// ) {
//     for (octree, cell_addresses) in query {
//         for entity in octree.cells.iter() {
//             if let Ok(cell) = cells.get(entity) {
//                 let cell_type = cell.get_cell_type();
//                 match cell_type {
//                     CellType::Branch => {
//                         for subcell_index in SubCellIndex::iter() {
//                             let child_address = cell.get_address().get_child_address(subcell_index);
//                             let child_cell_entity = cell_addresses.get(child_address);
//                             if let Some(cell) = cells.get(child_cel_entity) {
//                                 tessellation_traversal(cell, mesh);
//                             }
//                         }
//                     }
//                     CellType::Leaf => {
//                         for component in cell.get_componnets_mut().iter_mut() {
//                             tessellate_component(shape_surface, cms_query, mesh, component);
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
//
// pub fn tessellate_component(
//     shape_surface: Res<ShapeSurface>,
//     cms_query: Query<&CMSMeshInfo>,
//     mesh: Query<&mut Mesh>,
//     component: &mut Vec<usize>,
// ) {
//     let mut centro_id = Vertex::new();
//
//     let num_of_indices = component.len();
//
//     assert!(num_of_indices >= 3);
//
//     if num_of_indices == 3 {
//         make_tri(mesh, component);
//     } else if num_of_indices > 3 {
//         let mut centro_id_pos = Vector3::new(0.0, 0.0, 0.0);
//         let mut centro_id_normal = Vector3::new(0.0, 0.0, 0.0);
//
//         for comp in component.iter() {
//             let vertex = &cms_query.vertices[*comp];
//             centro_id_pos += vertex.get_position();
//             centro_id_normal += vertex.get_normals();
//         }
//
//         let mut med_vertex = Vector3::new(
//             centro_id_pos.x / num_of_indices as f32,
//             centro_id_pos.y / num_of_indices as f32,
//             centro_id_pos.z / num_of_indices as f32,
//         );
//
//         if self.snap_centro_id {
//             let med_val = self
//                 .sample_fn
//                 .get_value(med_vertex.x, med_vertex.y, med_vertex.z);
//
//             let med_dimension = self.offset / 2.0;
//
//             let mut med_gradient = Vector3::new(0.0, 0.0, 0.0);
//
//             CMS::find_gradient_with_value(
//                 self,
//                 &mut med_gradient,
//                 &med_dimension,
//                 &med_vertex,
//                 med_val,
//             );
//
//             // 沿着反方向偏移
//             med_vertex += -med_gradient * med_val;
//         }
//
//         centro_id.set_position(&med_vertex);
//
//         centro_id_normal.normalize_mut();
//
//         centro_id.set_normals(&centro_id_normal);
//
//         self.vertices.push(centro_id);
//
//         component.push(self.vertices.len() - 1);
//
//         CMS::make_tri_fan(mesh, component);
//     }
// }
//
// /// 3d梯度等价于2d的法线, 3d梯度等于2d方向上最快的变化方向，也就是法线
// /// 4d梯度等价于3d的法线, 4d梯度等于3d方向上最快的变化方向，也就是法线
// pub fn find_gradient_with_value(
//     shape_surface: Res<ShapeSurface>,
//     cms_query: Query<&CMSMeshInfo>,
//     normal: &mut Vector3<f32>,
//     dimensions: &Vector3<f32>,
//     position: &Vector3<f32>,
//     value: f32,
// ) {
//     let dx = self
//         .sample_fn
//         .get_value(position.x + dimensions.x, position.y, position.z);
//
//     let dy = self
//         .sample_fn
//         .get_value(position.x, position.y + dimensions.y, position.z);
//
//     let dz = self
//         .sample_fn
//         .get_value(position.x, position.y, position.z + dimensions.z);
//
//     *normal = Vector3::new(dx - value, dy - value, dz - value);
//
//     normal.normalize_mut();
// }
//
// pub fn make_tri(mesh: Query<&Mesh>, component: Query<(&CMSMeshInfo)>) {
//     for i in 0..3 {
//         mesh.get_indices_mut().push(component[i] as u32);
//     }
// }
//
// // 扇形三角面
// pub fn make_tri_fan(mesh: Query<&mut Mesh>, component: &Vec<usize>) {
//     for i in 0..(component.len() - 2) {
//         mesh.get_indices_mut()
//             .push(component[component.len() - 1] as u32);
//         mesh.get_indices_mut().push(component[i] as u32);
//         mesh.get_indices_mut().push(component[i + 1] as u32);
//     }
//
//     mesh.get_indices_mut()
//         .push(component[component.len() - 1] as u32);
//     mesh.get_indices_mut()
//         .push(component[component.len() - 2] as u32);
//     mesh.get_indices_mut().push(component[0] as u32);
// }
