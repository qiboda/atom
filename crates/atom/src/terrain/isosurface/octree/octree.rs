use bevy::{prelude::*, utils::HashMap};

use strum::{EnumCount, IntoEnumIterator};

use crate::terrain::{
    isosurface::{
        octree::{
            bundle::CellBundle,
            cell::{Cell, CellMeshInfo, CellType},
            face::{FaceType, Faces},
            tables::FaceIndex,
        },
        sample::surface_sampler::SurfaceSampler,
        surface::shape_surface::ShapeSurface,
        BuildOctreeState, IsosurfaceExtractionState,
    },
    settings::TerrainSettings,
};

use super::{
    address::VoxelAddress,
    def::COMPLEX_SURFACE_THRESHOLD,
    tables::{EdgeIndex, SubCellIndex, VertexPoint, EDGE_DIRECTION, EDGE_VERTICES},
};

#[derive(Debug, Component, Default)]
pub struct OctreeCellAddress {
    pub cell_addresses: HashMap<VoxelAddress, Entity>,
}

#[derive(Debug, Component, Default)]
pub struct Octree {
    pub cells: Vec<Entity>,

    pub leaf_cells: Vec<Entity>,

    pub transit_face_cells: Vec<Entity>,
}

// #[derive(Component)]
// struct BuildOctreeTask(Task<()>);
//
/// This system generates tasks simulating computationally intensive
/// work that potentially spans multiple frames/ticks. A separate
/// system, [`handle_tasks`], will poll the spawned tasks on subsequent
/// frames/ticks, and use the results to spawn cubes
// fn spawn_tasks(mut commands: Commands) {}
//
// fn handle_tasks(
//     mut commands: Commands,
//     mut transform_tasks: Query<(Entity, &mut BuildOctreeTask)>,
// ) {
//     for (entity, mut task) in &mut transform_tasks {
//         if let Some(transform) = future::block_on(future::poll_once(&mut task.0)) {}
//     }
// }

pub fn make_octree_structure(
    commands: ParallelCommands,
    shape_surface: Res<ShapeSurface>,
    terrain_settings: Res<TerrainSettings>,
    mut octree_query: Query<(
        Entity,
        &mut Octree,
        &mut OctreeCellAddress,
        &mut SurfaceSampler,
        &mut IsosurfaceExtractionState,
    )>,
) {
    // let thread_pool = AsyncComputeTaskPool::get();
    octree_query.par_iter_mut().for_each_mut(
        |(
            octree_entity,
            mut octree,
            mut octree_cell_address,
            mut surface_sampler,
            mut isosurface_extract_state,
        )| {
            if let IsosurfaceExtractionState::BuildOctree(BuildOctreeState::Build) =
                *isosurface_extract_state
            {
                info_span!("make_octree_structure");
                info!("make_octree_structure");
                debug!("make_structure");

                // let task = thread_pool.spawn(async move {
                let c000 = UVec3::new(0, 0, 0);

                let voxel_num = terrain_settings.get_chunk_voxel_num();
                let voxel_num = UVec3::splat(voxel_num);

                // todo: check is branch or leat cell.....
                let mut address = VoxelAddress::new();
                address.set(
                    VoxelAddress { raw_address: 1 },
                    SubCellIndex::LeftBottomBack,
                );

                let vertex_points = acquire_cell_info(c000, voxel_num);
                commands.command_scope(|mut command| {
                    let entity = command
                        .spawn(CellBundle {
                            cell: Cell::new(CellType::Branch, address, vertex_points),
                            faces: Faces::new(0, FaceType::BranchFace),
                            cell_mesh_info: CellMeshInfo::default(),
                        })
                        .id();

                    let mut octree_command = command.get_entity(octree_entity).unwrap();
                    octree_command.add_child(entity);
                    octree.cells.push(entity);
                    octree_cell_address.cell_addresses.insert(address, entity);
                });

                let voxel_num = voxel_num.x >> 1;
                let voxel_num = UVec3::splat(voxel_num);

                subdivide_cell(
                    &commands,
                    octree_entity,
                    &mut octree,
                    address,
                    c000,
                    voxel_num,
                    &mut surface_sampler,
                    &mut octree_cell_address,
                    &shape_surface,
                );

                debug!(
                    "cell num: {} and leaf cell num: {}",
                    octree.cells.len(),
                    octree.leaf_cells.len()
                );

                *isosurface_extract_state =
                    IsosurfaceExtractionState::BuildOctree(BuildOctreeState::MarkTransitFace);
            }
        },
    );
}

fn acquire_cell_info(c000: UVec3, voxel_num: UVec3) -> [UVec3; VertexPoint::COUNT] {
    let mut pt_indices = [UVec3::new(0, 0, 0); VertexPoint::COUNT];

    debug_assert!(voxel_num != UVec3::ZERO);

    {
        pt_indices[0] = UVec3::new(c000.x, c000.y, c000.z);
        pt_indices[1] = UVec3::new(c000.x, c000.y, c000.z + voxel_num.z);
        pt_indices[2] = UVec3::new(c000.x, c000.y + voxel_num.y, c000.z);
        pt_indices[3] = UVec3::new(c000.x, c000.y + voxel_num.y, c000.z + voxel_num.z);
        pt_indices[4] = UVec3::new(c000.x + voxel_num.x, c000.y, c000.z);
        pt_indices[5] = UVec3::new(c000.x + voxel_num.x, c000.y, c000.z + voxel_num.z);
        pt_indices[6] = UVec3::new(c000.x + voxel_num.x, c000.y + voxel_num.y, c000.z);
        pt_indices[7] = UVec3::new(
            c000.x + voxel_num.x,
            c000.y + voxel_num.y,
            c000.z + voxel_num.z,
        );
    }

    debug!("pt_indices: {:?}", pt_indices);

    pt_indices
}

fn subdivide_cell(
    commands: &ParallelCommands,
    octree_entity: Entity,
    octree: &mut Octree,
    parent_address: VoxelAddress,
    parent_c000: UVec3,
    voxel_num: UVec3,
    sample_info: &mut SurfaceSampler,
    cell_address: &mut OctreeCellAddress,
    shape_surface: &Res<ShapeSurface>,
) {
    // debug!("subdivide_cell: voxle num {}", voxel_num);

    for (i, subcell_index) in SubCellIndex::iter().enumerate() {
        let c000 = UVec3::new(
            parent_c000.x + voxel_num.x * ((i >> 2) & 1) as u32,
            parent_c000.y + voxel_num.y * ((i >> 1) & 1) as u32,
            parent_c000.z + voxel_num.z * (i & 1) as u32,
        );

        let vertex_points = acquire_cell_info(c000, voxel_num);
        let mut address = VoxelAddress::new();
        address.set(parent_address, subcell_index);

        let mut branch_type = CellType::Branch;
        if check_for_subdivision(sample_info, &vertex_points, shape_surface) {
            debug!("check_for_subdivision can subdivice cell");
            let voxel_num = voxel_num.x >> 1;
            let voxel_num = UVec3::splat(voxel_num);
            if voxel_num.x == 0 {
                // todo: 如此，如果不是在表面，就会忽略cell，这是否正确？
                if check_for_surface(&vertex_points, sample_info, &shape_surface) {
                    debug!("check_for_surface success: {:?}", vertex_points);
                    branch_type = CellType::Leaf;
                } else {
                    debug!("check_for_surface fail: {:?}", vertex_points);
                }
            } else {
                subdivide_cell(
                    commands,
                    octree_entity,
                    octree,
                    address,
                    c000,
                    voxel_num,
                    sample_info,
                    cell_address,
                    shape_surface,
                );
            }
        } else {
            // todo: 如此，如果不是在表面，就会忽略cell，这是否正确？
            if check_for_surface(&vertex_points, sample_info, &shape_surface) {
                debug!("check_for_surface success: {:?}", vertex_points);
                branch_type = CellType::Leaf;
            } else {
                debug!("check_for_surface fail: {:?}", vertex_points);
            }
        }

        let face_type = match branch_type {
            CellType::Branch => FaceType::BranchFace,
            CellType::Leaf => FaceType::LeafFace,
        };

        commands.command_scope(|mut command| {
            let entity = command
                .spawn(CellBundle {
                    cell: Cell::new(branch_type, address, vertex_points),
                    faces: Faces::new(0, face_type),
                    cell_mesh_info: CellMeshInfo::default(),
                })
                .id();

            let mut octree_command = command.get_entity(octree_entity).unwrap();
            octree_command.add_child(entity);
            octree.cells.push(entity);
            if branch_type == CellType::Leaf {
                octree.leaf_cells.push(entity);
            }
            cell_address.cell_addresses.insert(address, entity);
        });

        // debug!(
        //     "subdivide_cell: cell: {:?}",
        //     cell.borrow().get_corner_sample_index()
        // );
        //
    }
}

// 检查是否在表面
fn check_for_surface(
    vertex_points: &[UVec3; 8],
    sample_info: &mut SurfaceSampler,
    shape_surface: &Res<ShapeSurface>,
) -> bool {
    // 8个顶点中有几个在内部
    let mut inside = 0;
    for i in 0..8 {
        let value = sample_info.get_value_from_vertex_address(vertex_points[i], shape_surface);
        debug!(
            "inside value: {}, vertex_points: {}, world_offset: {}",
            value, vertex_points[i], sample_info.world_offset
        );
        if value < 0.0 {
            inside += 1;
        }
    }

    debug!(
        "check for surface: vertex_points: {:?}, inside: {}",
        vertex_points, inside
    );
    inside != 0 && inside != 8
}

fn check_for_subdivision(
    sample_info: &mut SurfaceSampler,
    vertex_points: &[UVec3; 8],
    shape_surface: &Res<ShapeSurface>,
) -> bool {
    let edge_ambiguity_result = check_for_edge_ambiguity(sample_info, vertex_points, shape_surface);
    debug!("check_for_edge_ambiguity_result: {edge_ambiguity_result}");
    if edge_ambiguity_result {
        return true;
    }

    let complex_surface_result =
        check_for_complex_surface(sample_info, vertex_points, shape_surface);
    debug!("check_for_complex_surface: {complex_surface_result}");
    return complex_surface_result;
}

/// 检测是否(坐标位置)平坦
fn check_for_edge_ambiguity(
    sample_info: &mut SurfaceSampler,
    vertex_points: &[UVec3; 8],
    shape_surface: &Res<ShapeSurface>,
) -> bool {
    // debug!("check_for_edge_ambiguity");
    let mut edge_ambiguity = false;

    for (i, _edge_index) in EdgeIndex::iter().enumerate() {
        let vertex_index_0 = EDGE_VERTICES[i][0] as usize;
        let vertex_index_1 = EDGE_VERTICES[i][1] as usize;

        let edge_direction = EDGE_DIRECTION[i];

        // debug!("edge_direction: {:?}", edge_direction);

        // left coord
        let point_0 = vertex_points[vertex_index_0];
        // right coord
        let point_1 = vertex_points[vertex_index_1];

        // debug!("point0: {:?} point1: {:?}", point_0, point_1);

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

            // debug!(
            //     "left: {} => {} right: {:?} => {}",
            //     index,
            //     sample_info.get_value_from_vertex_address(index, shape_surface),
            //     point_1,
            //     sample_info.get_value_from_vertex_address(point_1, shape_surface)
            // );
            debug_assert!(
                sample_info
                    .get_pos_from_vertex_address(index, shape_surface)
                    .x
                    <= sample_info
                        .get_pos_from_vertex_address(point_1, shape_surface)
                        .x
                    && sample_info
                        .get_pos_from_vertex_address(index, shape_surface)
                        .y
                        <= sample_info
                            .get_pos_from_vertex_address(point_1, shape_surface)
                            .y
                    && sample_info
                        .get_pos_from_vertex_address(index, shape_surface)
                        .z
                        <= sample_info
                            .get_pos_from_vertex_address(point_1, shape_surface)
                            .z
            );

            // debug!(
            //     "prev_point: {} value: {}, index: {} value: {}",
            //     prev_point,
            //     sample_info.get_value_from_vertex_address(prev_point, shape_surface),
            //     index,
            //     sample_info.get_value_from_vertex_address(index, shape_surface)
            // );
            // if the sign of the value at the previous point is different from the sign of the value at the current point,
            // then there is an edge ambiguity
            if sample_info.get_value_from_vertex_address(prev_point, shape_surface)
                * sample_info.get_value_from_vertex_address(index, shape_surface)
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
    sample_info: &mut SurfaceSampler,
    vertex_points: &[UVec3; 8],
    shape_surface: &Res<ShapeSurface>,
) -> bool {
    // debug!("check_for_complex_surface");
    let mut complex_surface = false;

    'outer: for i in 0..7 {
        let point_0 = vertex_points[i];

        let mut gradient_point_0 = Default::default();
        find_gradient(&mut gradient_point_0, &point_0, sample_info, shape_surface);

        for j in 1..8 {
            let point_1 = vertex_points[j];

            let mut gradient_point_1 = Default::default();
            find_gradient(&mut gradient_point_1, &point_1, sample_info, shape_surface);

            debug!(
                "point_0 {} gradient_point_0: {:?}, point_1 {} gradient_point_1: {:?} value {} < or not {}",
                point_0, gradient_point_0, point_1, gradient_point_1, gradient_point_0.dot(gradient_point_1), COMPLEX_SURFACE_THRESHOLD
            );
            debug!(
                "point 0 value:{} point 1 value:{}, ",
                sample_info.get_value_from_vertex_address(point_0, shape_surface),
                sample_info.get_value_from_vertex_address(point_1, shape_surface)
            );
            if gradient_point_0.dot(gradient_point_1) < COMPLEX_SURFACE_THRESHOLD {
                debug!("is complex surface");
                complex_surface = true;
                break 'outer;
            }
        }
    }

    complex_surface
}

fn find_gradient(
    gradient: &mut Vec3,
    point: &UVec3,
    sample_info: &mut SurfaceSampler,
    shape_surface: &Res<ShapeSurface>,
) {
    let mut dimensions = Vec3::new(0.0, 0.0, 0.0);

    // why use half offset?
    for i in 0..3 {
        dimensions[i] = sample_info.voxel_size[i] / 2.0;
    }

    let dx = sample_info.get_value_from_vertex_offset(
        *point,
        Vec3::new(dimensions.x, 0.0, 0.0),
        shape_surface,
    );
    let dy = sample_info.get_value_from_vertex_offset(
        *point,
        Vec3::new(0.0, dimensions.y, 0.0),
        shape_surface,
    );
    let dz = sample_info.get_value_from_vertex_offset(
        *point,
        Vec3::new(0.0, 0.0, dimensions.z),
        shape_surface,
    );
    let val = sample_info.get_value_from_vertex_address(*point, shape_surface);

    debug!("dx dy dz: {} {} {} val: {}", dz, dy, dz, val);
    *gradient = Vec3::new(dx - val, dy - val, dz - val).normalize();
}

pub fn mark_transitional_faces(
    mut cell_faces: Query<(&mut Cell, &mut Faces)>,
    mut query: Query<(
        &mut Octree,
        &OctreeCellAddress,
        &mut IsosurfaceExtractionState,
    )>,
) {
    query
        // .par_iter_mut()
        .for_each_mut(|(mut octree, cell_address, mut state)| {
            if let IsosurfaceExtractionState::BuildOctree(BuildOctreeState::MarkTransitFace) =
                *state
            {
                if octree.cells.len() == 0 {
                    return;
                }

                if let Err(_) = cell_faces.get(octree.cells[0]) {
                    return;
                }

                info_span!("mark_transitional_faces");
                info!(
                    "mark_transitional_faces: cell num: {} leaf cells: {}, transitional_cells: {}",
                    octree.cells.len(),
                    octree.leaf_cells.len(),
                    octree.transit_face_cells.len()
                );
                let mut transitional_cells = Vec::new();

                let mut face_branch_num = 0;
                let mut face_leaf_num = 0;
                for entity in octree.cells.iter() {
                    if let Ok((_cell, faces)) = cell_faces.get(*entity) {
                        for face_index in FaceIndex::iter() {
                            match faces.get_face(face_index).get_face_type() {
                                FaceType::BranchFace => {
                                    face_branch_num += 1;
                                }
                                FaceType::LeafFace => {
                                    face_leaf_num += 1;
                                }
                                FaceType::TransitFace => debug_assert!(false),
                            }
                        }
                    }
                }
                debug!(
                    "branch face: {}, leaf face: {}",
                    face_branch_num, face_leaf_num
                );

                for entity in octree.leaf_cells.iter() {
                    debug!("entity: {:?}", entity);
                    let mut all_neighbour_cell_entity = [
                        Entity::PLACEHOLDER,
                        Entity::PLACEHOLDER,
                        Entity::PLACEHOLDER,
                        Entity::PLACEHOLDER,
                        Entity::PLACEHOLDER,
                        Entity::PLACEHOLDER,
                    ];

                    let mut all_neighbour_face_index = [
                        FaceIndex::Left,
                        FaceIndex::Left,
                        FaceIndex::Left,
                        FaceIndex::Left,
                        FaceIndex::Left,
                        FaceIndex::Left,
                    ];

                    if let Ok((leaf_cell, mut faces)) = cell_faces.get_mut(*entity) {
                        debug_assert!(leaf_cell.get_cell_type() == &CellType::Leaf);

                        for (i, face_index) in FaceIndex::iter().enumerate() {
                            debug!("entity: {:?} face index: {:?}", entity, face_index);
                            let face = faces.get_face_mut(face_index);
                            debug_assert!(face.get_face_type() == &FaceType::LeafFace);

                            let (neighbour_cell_address, neighbour_face_index) =
                                leaf_cell.get_twin_face_address(face_index);
                            all_neighbour_face_index[i] = neighbour_face_index;

                            if let Some(neighbour_cell_entity) =
                                cell_address.cell_addresses.get(&neighbour_cell_address)
                            {
                                all_neighbour_cell_entity[i] = *neighbour_cell_entity;
                            } else {
                                debug!("get cell fail from cell address!");
                            }
                        }
                    } else {
                        debug!("get cell fail!");
                    }

                    let mut set_transit_face = [false, false, false, false, false, false];

                    for (i, entity) in all_neighbour_cell_entity.iter().enumerate() {
                        let neighbour_face_index = all_neighbour_face_index[i];
                        debug!(
                            "entity: {:?} neighbour face index: {:?}",
                            entity, neighbour_face_index
                        );
                        if let Ok((_neighbour_cell, neighbour_faces)) = cell_faces.get(*entity) {
                            if neighbour_faces
                                .get_face(neighbour_face_index)
                                .get_face_type()
                                == &FaceType::BranchFace
                            {
                                set_transit_face[i] = true;
                                debug!("set_transit_face true {}", i);
                            } else {
                                debug!(
                                    "entity: {:?} neighbour face index: {:?} face type: {:?}",
                                    entity,
                                    neighbour_face_index,
                                    neighbour_faces
                                        .get_face(neighbour_face_index)
                                        .get_face_type()
                                );
                            }
                        } else {
                            debug!("get neighbour cell fail from entity!");
                        }
                    }

                    let mut b_set = false;
                    if let Ok((leaf_cell, mut faces)) = cell_faces.get_mut(*entity) {
                        for (i, set) in set_transit_face.iter().enumerate() {
                            debug_assert!(leaf_cell.get_cell_type() == &CellType::Leaf);
                            if *set {
                                let face = faces.get_face_mut(FaceIndex::from_repr(i).unwrap());
                                face.set_face_type(FaceType::TransitFace);
                                b_set = true;
                                debug!("set_transit_face transit face {}", i);
                            }
                        }
                    } else {
                        debug!("get leaf cell fail from entity!");
                    }

                    if b_set {
                        transitional_cells.push(*entity);
                        debug_assert!(octree.leaf_cells.contains(entity));
                    }
                }
                octree.transit_face_cells = transitional_cells;

                debug!(
                    "transit_face_cells num: {}",
                    octree.transit_face_cells.len()
                );
                *state = IsosurfaceExtractionState::Extract;
            }
        });
}
