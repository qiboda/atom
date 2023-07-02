use std::collections::HashMap;

use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    octree::{
        bundle::CellBundle,
        cell::{Cell, CellMeshInfo, CellType},
        def::MAX_OCTREE_RES,
        face::FaceType,
        tables::FaceIndex,
    },
    sample::sample_info::SampleInfo,
};

use strum::{EnumCount, IntoEnumIterator};

use super::{
    address::Address,
    def::COMPLEX_SURFACE_THRESHOLD,
    face::Faces,
    tables::{EdgeIndex, SubCellIndex, VertexPoint, EDGE_DIRECTION, EDGE_VERTICES},
};

#[derive(Debug, Component, Default)]
pub struct OctreeCellAddress {
    pub cell_addresses: HashMap<Address, Entity>,
}

#[derive(Debug, Component, Default)]
pub struct Octree {
    pub cells: Vec<Entity>,

    pub leaf_cells: Vec<Entity>,

    pub transit_face_cells: Vec<Entity>,
}

pub fn make_octree_structure(
    mut commands: Commands,
    sample_info: Query<&SampleInfo>,
    octree_query: Query<(&mut Octree, &mut OctreeCellAddress), Added<Octree>>,
) {
    for (mut octree, mut cell_address) in octree_query.iter_mut() {
        info!("make_structure");
        let c000 = Vector3::new(0, 0, 0);

        // todo: check is branch or leat cell.....
        let mut address = Address::new();
        address.set(Address::new(), SubCellIndex::LeftBottomBack);

        let vertex_points = acquire_cell_info(c000, &sample_info);
        let entity = commands
            .spawn(CellBundle {
                cell: Cell::new(
                    0,
                    CellType::Branch,
                    address,
                    c000,
                    sample_info.samples_size - Vector3::new(1, 1, 1),
                    vertex_points,
                ),
                faces: Faces::new(0, FaceType::BranchFace),
                cell_mesh_info: CellMeshInfo::default(),
            })
            .id();

        octree.cells.push(entity);
        cell_address.cell_addresses.insert(address, entity);

        subdivide_cell(
            &mut commands,
            &mut octree,
            &mut address,
            c000,
            &sample_info,
            vertex_points,
            &mut cell_address,
        );
    }
}

fn acquire_cell_info(c000: Vector3<usize>, sample_info: &SampleInfo) {
    let mut pt_indices = [Vector3::new(0, 0, 0); VertexPoint::COUNT];

    {
        pt_indices[0] = Vector3::new(c000.x, c000.y, c000.z);
        pt_indices[1] = Vector3::new(c000.x, c000.y, c000.z + sample_info.offsets.z);
        pt_indices[2] = Vector3::new(c000.x, c000.y + sample_info.offsets.y, c000.z);
        pt_indices[3] = Vector3::new(
            c000.x,
            c000.y + sample_info.offsets.y,
            c000.z + sample_info.offsets.z,
        );
        pt_indices[4] = Vector3::new(c000.x + sample_info.offsets.x, c000.y, c000.z);
        pt_indices[5] = Vector3::new(
            c000.x + sample_info.offsets.x,
            c000.y,
            c000.z + sample_info.offsets.z,
        );
        pt_indices[6] = Vector3::new(
            c000.x + sample_info.offsets.x,
            c000.y + sample_info.offsets.y,
            c000.z,
        );
        pt_indices[7] = Vector3::new(
            c000.x + sample_info.offsets.x,
            c000.y + sample_info.offsets.y,
            c000.z + sample_info.offsets.z,
        );

        // todo: 排除右边缘??????
        for pt_index in pt_indices.iter_mut() {
            pt_index.x = pt_index.x.clamp(0, sample_info.samples_size.x - 1);
            pt_index.y = pt_index.y.clamp(0, sample_info.samples_size.y - 1);
            pt_index.z = pt_index.z.clamp(0, sample_info.samples_size.z - 1);
        }
    }

    pt_indices
}

fn subdivide_cell(
    mut commands: &mut Commands,
    octree: &mut Octree,
    parent_address: Address,
    parent_c000: Vector3<usize>,
    sample_info: &SampleInfo,
    parent_vertex_points: &[Vector3<usize>; 8],
    cell_address: &mut OctreeCellAddress,
) {
    let this_level = parent_address.get_level();
    if this_level >= MAX_OCTREE_RES {
        return;
    }

    // info!("subdivide_cell: this level: {}", this_level);

    let mut sample_size = Vector3::new(0, 0, 0);

    sample_size[0] = (sample_info.samples_size[0] - 1) >> this_level;
    sample_size[1] = (sample_info.samples_size[1] - 1) >> this_level;
    sample_size[2] = (sample_info.samples_size[2] - 1) >> this_level;

    // info!("subdivide_cell: sample size: {}", sample_size);

    for (i, subcell_index) in SubCellIndex::iter().enumerate() {
        let c000 = Vector3::new(
            parent_c000.x + sample_size.x * ((i >> 2) & 1),
            parent_c000.y + sample_size.y * ((i >> 1) & 1),
            parent_c000.z + sample_size.z * (i & 1),
        );

        let vertex_points = acquire_cell_info(c000, &sample_info);
        let address = Address::new().set(parent_address, subcell_index);

        let branch_type = CellType::Branch;
        if check_for_subdivision(c000, &sample_info) {
            subdivide_cell(
                commands,
                octree,
                address,
                c000,
                sample_info,
                vertex_points,
                cell_address,
            );
        } else {
            // todo: 如此，如果不是在表面，就会忽略cell，这是否正确？
            // info!("{this_level}:{i}: check_for_surface: {}", surface);
            if check_for_surface(c000, &sample_info, parent_vertex_points) {
                branch_type = CellType::Leaf;
            }
        }

        let face_type = match branch_type {
            CellType::Branch => FaceType::BranchFace,
            CellType::Leaf => FaceType::LeafFace,
        };

        let entity = commands
            .spawn(CellBundle {
                cell: Cell::new(
                    this_level + 1,
                    branch_type,
                    address,
                    c000,
                    sample_size,
                    vertex_points,
                ),
                faces: Faces::new(0, face_type),
                cell_mesh_info: CellMeshInfo::default(),
            })
            .id();

        octree.cells.push(entity);
        if branch_type == CellType::Leaf {
            octree.leaf_cells.push(entity);
        }
        cell_address.cell_addresses.insert(address, entity);

        // info!(
        //     "subdivide_cell: cell: {:?}",
        //     cell.borrow().get_corner_sample_index()
        // );
        //
    }
}

// 检查是否在表面
fn check_for_surface(
    address: Address,
    vertex_points: &[Vector3<usize>; 8],
    sample_info: &SampleInfo,
) -> bool {
    // 8个顶点中有几个在内部
    let mut inside = 0;
    for i in 0..8 {
        if sample_info.sample_data.get_value(
            vertex_points[i].x,
            vertex_points[i].y,
            vertex_points[i].z,
        ) < 0.0
        {
            inside += 1;
        }

        // if cell.borrow().get_address().get_formatted() == 57337070 {
        //     info!(
        //         "inside: {i} {} {}",
        //         inside,
        //         self.sample_data.get_value(
        //             pos_in_parent[i].x,
        //             pos_in_parent[i].y,
        //             pos_in_parent[i].z,
        //         )
        //     );
        // }
    }

    // if cell.borrow().get_address().get_formatted() == 57337070 {
    //     info!("inside: total {}", inside);
    // }
    //
    // info!("check_for_surface: inside: {}", inside);

    inside != 0 && inside != 8
}

fn check_for_subdivision(sample_info: &SampleInfo, vertex_points: &[Vector3<usize>; 8]) -> bool {
    check_for_edge_ambiguity(sample_info, vertex_points)
        || check_for_complex_surface(sample_info, vertex_points)
}

/// 检测是否(坐标位置)平坦
fn check_for_edge_ambiguity(sample_info: &SampleInfo, vertex_points: &[Vector3<usize>; 8]) -> bool {
    let mut edge_ambiguity = false;

    for (i, _edge_index) in EdgeIndex::iter().enumerate() {
        let vertex_index_0 = EDGE_VERTICES[i][0] as usize;
        let vertex_index_1 = EDGE_VERTICES[i][1] as usize;

        let edge_direction = EDGE_DIRECTION[i];

        // info!("edge_direction: {:?}", edge_direction);

        // left coord
        let point_0 = vertex_points[vertex_index_0];
        // right coord
        let point_1 = vertex_points[vertex_index_1];

        // info!("point0: {:?} point1: {:?}", point_0, point_1);

        // max right index
        let last_index = sample_info
            .sample_data
            .get_data_index(point_1.x, point_1.y, point_1.z);

        // last iter coord
        let mut prev_point = point_0;
        // iter coord
        let mut index = point_0;
        index[edge_direction as usize] += 1;

        // 以所在边的轴向为基准，迭代整个边
        // 是否可以优化,避免迭代次数过多。例如，二分法.
        loop {
            // 如果index的坐标大于point_1的坐标，说明已经迭代到了point_1的坐标，可以退出了
            if index[edge_direction as usize] > point_1[edge_direction as usize] {
                break;
            }

            assert!(
                sample_info
                    .sample_data
                    .get_data_index(index.x, index.y, index.z)
                    <= last_index
            );

            // if the sign of the value at the previous point is different from the sign of the value at the current point,
            // then there is an edge ambiguity
            if sample_info
                .sample_data
                .get_value(prev_point.x, prev_point.y, prev_point.z)
                * sample_info.sample_data.get_value(index.x, index.y, index.z)
                < 0.0
            {
                edge_ambiguity = true;
                break;
            }

            prev_point = index;
            index[edge_direction as usize] += 1;
        }
    }

    edge_ambiguity
}

// normal angle...
fn check_for_complex_surface(
    sample_info: &SampleInfo,
    vertex_points: &[Vector3<usize>; 8],
) -> bool {
    let mut complex_surface = false;

    'outer: for i in 0..7 {
        let point_0 = vertex_points[i];

        let mut gradient_point_0 = Default::default();
        find_gradient(&mut gradient_point_0, &point_0, sample_info);

        for j in 1..8 {
            let point_1 = vertex_points[j];

            let mut gradient_point_1 = Default::default();
            find_gradient(&mut gradient_point_1, &point_1, sample_info);

            if gradient_point_0.dot(&gradient_point_1) < COMPLEX_SURFACE_THRESHOLD {
                complex_surface = true;
                break 'outer;
            }
        }
    }

    complex_surface
}

fn find_gradient(gradient: &mut Vector3<f32>, point: &Vector3<usize>, sample_info: &SampleInfo) {
    let pos = sample_info.sample_data.get_pos(point.x, point.y, point.z);

    let mut dimensions = Vector3::new(0.0, 0.0, 0.0);

    // why use half offset?
    for i in 0..3 {
        dimensions[i] = sample_info.offsets[i] / 2.0;
    }

    let dx = sample_info
        .shape_surface
        .get_value(pos.x + dimensions.x, pos.y, pos.z);
    let dy = sample_info
        .shape_surface
        .get_value(pos.x, pos.y + dimensions.y, pos.z);
    let dz = sample_info
        .shape_surface
        .get_value(pos.x, pos.y, pos.z + dimensions.z);
    let val = sample_info.sample_data.get_value(point.x, point.y, point.z);

    *gradient = Vector3::new(dx - val, dy - val, dz - val);
    gradient.normalize_mut();
}

// fn populate_half_faces() {
//     info!("populate_half_faces");
//
//     let mut contact_cell_address = [
//         Address::new(),
//         Address::new(),
//         Address::new(),
//         Address::new(),
//         Address::new(),
//         Address::new(),
//     ];
//
//     let mut temp_neightbour_address = [vec![], vec![], vec![], vec![], vec![], vec![]];
//     for (i, _) in FaceIndex::iter().enumerate() {
//         temp_neightbour_address[i].resize(MAX_OCTREE_RES, None);
//     }
//
//     for cell in &self.cells {
//         for (i, face_index) in FaceIndex::iter().enumerate() {
//             for depth in (0..MAX_OCTREE_RES).rev() {
//                 // 得到对应层级的在父级的位置。
//                 let value = cell.borrow().get_address().get_raw()[depth];
//                 let axis = FACE_DIRECTION[i];
//                 match value {
//                     Some(v) => {
//                         temp_neightbour_address[i][depth] =
//                             Some(NEIGHBOUR_ADDRESS_TABLE[axis as usize][v as usize]);
//                     }
//                     None => {
//                         temp_neightbour_address[i][depth] = None;
//                     }
//                 }
//             }
//
//             contact_cell_address[i].populate_address(&temp_neightbour_address[i]);
//         }
//
//         for (i, face_index) in FaceIndex::iter().enumerate() {
//             let address_key = contact_cell_address[i].get_formatted();
//
//             let contact_cell = self.cell_addresses.get(&address_key);
//             if contact_cell.is_some() {
//                 // info!(
//                 //     "contact cell address: {} type: {:?}, cell address: {} type: {:?}",
//                 //     contact_cell_address[i].get_formatted(),
//                 //     contact_cell.unwrap().borrow().get_cell_type(),
//                 //     cell.borrow().get_address().get_formatted(),
//                 //     cell.borrow().get_cell_type(),
//                 // );
//                 assert!(
//                     contact_cell.unwrap().borrow().get_cur_subdiv_level()
//                         == cell.borrow().get_cur_subdiv_level()
//                 );
//                 // let neighbour_face_index = FACE_NEIGHBOUR[i];
//                 cell.borrow_mut()
//                     .set_neighbor(face_index, Some(contact_cell.unwrap().clone()));
//
//                 self.set_face_twins(contact_cell.unwrap().clone(), cell.clone(), face_index);
//             }
//         }
//
//         for address in contact_cell_address.iter_mut() {
//             address.reset()
//         }
//
//         for address in temp_neightbour_address.iter_mut() {
//             address.resize(MAX_OCTREE_RES, None);
//         }
//     }
// }
//
// fn set_face_twins(
//     &self,
//     contact_cell: Rc<RefCell<Cell>>,
//     cell: Rc<RefCell<Cell>>,
//     face_index: FaceIndex,
// ) {
//     assert!(contact_cell.borrow().get_cur_subdiv_level() == cell.borrow().get_cur_subdiv_level());
//     // assert!(contact_cell.borrow().get_cell_type() != &CellType::Leaf);
//
//     let val = FACE_TWIN_TABLE[face_index as usize][0];
//     let val_contact = FACE_TWIN_TABLE[face_index as usize][1];
//
//     cell.borrow_mut()
//         .get_face(val)
//         .borrow_mut()
//         .set_twin(contact_cell.borrow().get_face(val_contact).clone());
//
//     assert!(
//         contact_cell
//             .borrow()
//             .get_face(val_contact)
//             .borrow()
//             .get_face_type()
//             == cell.borrow().get_face(val).borrow().get_face_type()
//     );
//
//     contact_cell
//         .borrow_mut()
//         .get_face(val_contact)
//         .borrow_mut()
//         .set_twin(cell.borrow().get_face(val).clone());
//
//     let cell_2 = cell.borrow();
//     let face = cell_2.get_face(face_index);
//     let cell_2_face = face.borrow();
//     let id = cell_2_face.get_face_index();
//
//     let cell_2_face_twin = cell_2_face.get_twin();
//     cell_2_face_twin.clone().map(|x| {
//         let cell_2_face_twin = x.borrow();
//         cell_2_face_twin.get_twin().clone().map(|x| {
//             let id_2 = x.borrow().get_face_index();
//             assert!(id == id_2);
//         });
//     });
// }
//

pub fn mark_transitional_faces(
    cell_faces: Query<(&mut Cell, &mut Faces)>,
    query: Query<(&mut Octree, &OctreeCellAddress), Added<Octree>>,
) {
    info!("mark_transitional_faces");
    for (mut octree, cell_address) in query.iter() {
        for entity in octree.leaf_cells.iter() {
            if let Ok((leaf_cell, faces)) = cell_faces.get(*entity) {
                assert!(leaf_cell.get_cell_type() == &CellType::Leaf);

                for face_index in FaceIndex::iter() {
                    let mut face = faces.get_face_mut(face_index);
                    assert!(face.get_face_type() == &FaceType::LeafFace);

                    let (neighbour_cell_address, neighbour_face_index) =
                        leaf_cell.get_twin_face_address(face_index);
                    if let Some(neighbour_cell_entity) =
                        cell_address.cell_addresses.get(&neighbour_cell_address)
                    {
                        if let Ok((neighbour_cell, neighbour_faces)) =
                            cell_faces.get(*neighbour_cell_entity)
                        {
                            if neighbour_faces
                                .get_face(neighbour_face_index)
                                .get_face_type()
                                == &FaceType::BranchFace
                            {
                                face.set_face_type(FaceType::TransitFace);
                                octree.transit_face_cells.push(*entity);
                            }
                        }
                    }
                }
            }
        }
    }
}
