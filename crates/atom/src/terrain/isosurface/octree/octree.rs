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
    mut octree_query: Query<
        (
            &mut Octree,
            &mut OctreeCellAddress,
            &mut SurfaceSampler,
            &mut IsosurfaceExtractionState,
        ),
        Added<Octree>,
    >,
) {
    // let thread_pool = AsyncComputeTaskPool::get();
    octree_query.par_iter_mut().for_each_mut(
        |(
            mut octree,
            mut octree_cell_address,
            mut surface_sampler,
            mut isosurface_extract_state,
        )| {
            if let IsosurfaceExtractionState::BuildOctree(BuildOctreeState::Build) =
                *isosurface_extract_state
            {
                info!("make_octree_structure");
                commands.command_scope(|mut commands| {
                    info!("make_structure");

                    // let task = thread_pool.spawn(async move {
                    let c000 = UVec3::new(0, 0, 0);

                    let voxel_num = terrain_settings.get_chunk_voxel_num();
                    let voxel_num = UVec3::splat(voxel_num);

                    // todo: check is branch or leat cell.....
                    let mut address = VoxelAddress::new();
                    address.set(VoxelAddress::new(), SubCellIndex::LeftBottomBack);

                    let vertex_points = acquire_cell_info(c000, voxel_num);
                    let entity = commands
                        .spawn(CellBundle {
                            cell: Cell::new(CellType::Branch, address, vertex_points),
                            faces: Faces::new(0, FaceType::BranchFace),
                            cell_mesh_info: CellMeshInfo::default(),
                        })
                        .id();

                    octree.cells.push(entity);
                    octree_cell_address.cell_addresses.insert(address, entity);

                    subdivide_cell(
                        &mut commands,
                        &mut octree,
                        address,
                        c000,
                        voxel_num,
                        &mut surface_sampler,
                        &vertex_points,
                        &mut octree_cell_address,
                        &shape_surface,
                    );
                    // });
                });

                *isosurface_extract_state =
                    IsosurfaceExtractionState::BuildOctree(BuildOctreeState::MarkTransitFace);
            }
        },
    );
    // Spawn new entity and add our new task as a component
    // commands.spawn(BuildOctreeTask(task));
}

fn acquire_cell_info(c000: UVec3, voxel_num: UVec3) -> [UVec3; VertexPoint::COUNT] {
    let mut pt_indices = [UVec3::new(0, 0, 0); VertexPoint::COUNT];

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

        // // todo: 排除右边缘??????
        // for pt_index in pt_indices.iter_mut() {
        //     pt_index.x = pt_index.x.clamp(0, voxel_num.x - 1);
        //     pt_index.y = pt_index.y.clamp(0, voxel_num.y - 1);
        //     pt_index.z = pt_index.z.clamp(0, voxel_num.z - 1);
        // }
    }

    pt_indices
}

fn subdivide_cell(
    commands: &mut Commands,
    octree: &mut Octree,
    parent_address: VoxelAddress,
    parent_c000: UVec3,
    parent_voxel_num: UVec3,
    sample_info: &mut SurfaceSampler,
    parent_vertex_points: &[UVec3; 8],
    cell_address: &mut OctreeCellAddress,
    shape_surface: &Res<ShapeSurface>,
) {
    // info!("subdivide_cell: this level: {}", this_level);
    let voxel_num = parent_voxel_num.x >> 1;
    let voxel_num = UVec3::splat(voxel_num);

    if voxel_num.x == 0 {
        return;
    }

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
            subdivide_cell(
                commands,
                octree,
                address,
                c000,
                voxel_num,
                sample_info,
                &vertex_points,
                cell_address,
                shape_surface,
            );
        } else {
            // todo: 如此，如果不是在表面，就会忽略cell，这是否正确？
            // info!("{this_level}:{i}: check_for_surface: {}", surface);
            if check_for_surface(parent_vertex_points, sample_info, &shape_surface) {
                branch_type = CellType::Leaf;
            }
        }

        let face_type = match branch_type {
            CellType::Branch => FaceType::BranchFace,
            CellType::Leaf => FaceType::LeafFace,
        };

        let entity = commands
            .spawn(CellBundle {
                cell: Cell::new(branch_type, address, vertex_points),
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
    vertex_points: &[UVec3; 8],
    sample_info: &mut SurfaceSampler,
    shape_surface: &Res<ShapeSurface>,
) -> bool {
    // 8个顶点中有几个在内部
    let mut inside = 0;
    for i in 0..8 {
        if sample_info.get_value_from_vertex_address(vertex_points[i], shape_surface) < 0.0 {
            inside += 1;
        }
    }

    inside != 0 && inside != 8
}

fn check_for_subdivision(
    sample_info: &mut SurfaceSampler,
    vertex_points: &[UVec3; 8],
    shape_surface: &Res<ShapeSurface>,
) -> bool {
    check_for_edge_ambiguity(sample_info, vertex_points, shape_surface)
        || check_for_complex_surface(sample_info, vertex_points, shape_surface)
}

/// 检测是否(坐标位置)平坦
fn check_for_edge_ambiguity(
    sample_info: &mut SurfaceSampler,
    vertex_points: &[UVec3; 8],
    shape_surface: &Res<ShapeSurface>,
) -> bool {
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
                sample_info.get_value_from_vertex_address(index, shape_surface)
                    <= sample_info.get_value_from_vertex_address(point_1, shape_surface)
            );

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
    let mut complex_surface = false;

    'outer: for i in 0..7 {
        let point_0 = vertex_points[i];

        let mut gradient_point_0 = Default::default();
        find_gradient(&mut gradient_point_0, &point_0, sample_info, shape_surface);

        for j in 1..8 {
            let point_1 = vertex_points[j];

            let mut gradient_point_1 = Default::default();
            find_gradient(&mut gradient_point_1, &point_1, sample_info, shape_surface);

            if gradient_point_0.dot(gradient_point_1) < COMPLEX_SURFACE_THRESHOLD {
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
    query.for_each_mut(|(mut octree, cell_address, mut state)| {
        if let IsosurfaceExtractionState::BuildOctree(BuildOctreeState::MarkTransitFace) = *state {
            info!("mark_transitional_faces");
            let mut transitional_cells = Vec::new();

            for entity in octree.leaf_cells.iter() {
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
                    assert!(leaf_cell.get_cell_type() == &CellType::Leaf);

                    for (i, face_index) in FaceIndex::iter().enumerate() {
                        let face = faces.get_face_mut(face_index);
                        assert!(face.get_face_type() == &FaceType::LeafFace);

                        let (neighbour_cell_address, neighbour_face_index) =
                            leaf_cell.get_twin_face_address(face_index);
                        all_neighbour_face_index[i] = neighbour_face_index;

                        if let Some(neighbour_cell_entity) =
                            cell_address.cell_addresses.get(&neighbour_cell_address)
                        {
                            all_neighbour_cell_entity[i] = *neighbour_cell_entity;
                        }
                    }
                }

                let mut set_transit_face = [false, false, false, false, false, false];

                for (i, entity) in all_neighbour_cell_entity.iter().enumerate() {
                    let neighbour_face_index = all_neighbour_face_index[i];
                    if let Ok((_neighbour_cell, neighbour_faces)) = cell_faces.get(*entity) {
                        if neighbour_faces
                            .get_face(neighbour_face_index)
                            .get_face_type()
                            == &FaceType::BranchFace
                        {
                            set_transit_face[i] = true;
                        }
                    }
                }

                let mut b_set = false;
                if let Ok((leaf_cell, mut faces)) = cell_faces.get_mut(*entity) {
                    for (i, set) in set_transit_face.iter().enumerate() {
                        assert!(leaf_cell.get_cell_type() == &CellType::Leaf);
                        if *set {
                            let face = faces.get_face_mut(FaceIndex::from_repr(i).unwrap());
                            face.set_face_type(FaceType::TransitFace);
                            b_set = true;
                        }
                    }
                }

                if b_set {
                    transitional_cells.push(*entity);
                }
            }
            octree.transit_face_cells = transitional_cells;

            *state = IsosurfaceExtractionState::Extract;
        }
    });
}
