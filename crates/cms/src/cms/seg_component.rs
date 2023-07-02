// pub fn cubical_marching_sqaures_algorithm() {
//     generate_segments(cells, query, query_add)
//     edit_transitional_face();
//
//     trace_comonent();
// }

use std::ops::Range;

use bevy::prelude::*;
use nalgebra::Vector3;

use strum::EnumCount;

use crate::{
    octree::{
        cell::{Cell, CellMeshInfo},
        face::{Face, FaceType, Faces},
        octree::{Octree, OctreeCellAddress},
        point::Point,
        strip::Strip,
        tables::{
            EdgeDirection, Face2DEdge, Face2DVertex, FaceIndex, EDGE_MAP, FACE_2_SUBCELL,
            FACE_VERTEX, VERTEX_MAP,
        },
        vertex::Vertex,
    },
    sample::sample_info::SampleInfo,
    surface::shape_surface::ShapeSurface,
};

use super::cms::CMSMeshInfo;

/// 生成每个面的连线，以及边的顶点的位置信息。
pub fn generate_segments(
    shape_surface: Res<ShapeSurface>,
    cells: Query<(&Cell, &mut CellMeshInfo, &mut Faces)>,
    query: Query<(&Octree, &OctreeCellAddress)>,
    sample_query: Query<(&mut CMSMeshInfo, &SampleInfo)>,
) {
    let (mesh_info, sample_info) = sample_query.single();
    for (octree, address) in query.iter() {
        for entity in octree.leaf_cells.iter() {
            if let Ok((cell, mut cell_mesh_info, mut faces)) = cells.get(*entity) {
                let mut indices = [Vector3::zeros(); Face2DVertex::COUNT];
                for face_index in FaceIndex::iter() {
                    for (i, face_vertex) in Face2DVertex::iter().enumerate() {
                        let vertex_pos = FACE_VERTEX[face_index as usize][face_vertex as usize];
                        indices[i] = cell.get_corner_sample_index()[vertex_pos as usize];
                    }

                    let face = faces.get_face(face_index);
                    face.get_strips_mut().resize(2, Strip::default());
                    make_face_segments(
                        &indices,
                        face,
                        &mut cell_mesh_info,
                        &mut mesh_info,
                        &sample_info,
                        shape_surface,
                    );
                }
            }
        }
    }
}

// make segments in a face.
fn make_face_segments(
    indices: &[Vector3<usize>; 4],
    face: &Face,
    cell_mesh_info: &mut CellMeshInfo,
    mesh_info: &mut CMSMeshInfo,
    sample_info: &SampleInfo,
    shape_surface: Res<ShapeSurface>,
) {
    let edges = (0..4).fold(0, |acc, i| {
        acc | if sample_info
            .sample_data
            .get_value(indices[i].x, indices[i].y, indices[i].z)
            < 0.0
        {
            1 << i
        } else {
            0
        }
    });

    let e0a = EDGE_MAP[edges as usize][0][0];
    let e0b = EDGE_MAP[edges as usize][0][1];

    if e0a.is_some() {
        make_strip(
            e0a,
            e0b,
            indices,
            face.clone(),
            0,
            cell_mesh_info,
            mesh_info,
            sample_info,
            shape_surface,
        );
    }

    let e1a = EDGE_MAP[edges as usize][1][0];
    let e1b = EDGE_MAP[edges as usize][1][1];

    if e1a.is_some() {
        make_strip(
            e1a,
            e1b,
            indices,
            face.clone(),
            1,
            cell_mesh_info,
            mesh_info,
            sample_info,
            shape_surface,
        );
    }
}

pub fn make_strip(
    edge0: Option<Face2DEdge>,
    edge1: Option<Face2DEdge>,
    indices: &[Vector3<usize>; 4],
    face: &Face,
    strip_index: usize,
    cell_mesh_info: &mut CellMeshInfo,
    mesh_info: &mut CMSMeshInfo,
    sample_info: &SampleInfo,
    shape_surface: Res<ShapeSurface>,
) {
    assert!(edge0.is_some() && edge1.is_some());

    let mut s = Strip::new(edge0, edge1);

    populate_strip(
        &mut s,
        indices,
        0,
        cell_mesh_info,
        mesh_info,
        sample_info,
        shape_surface,
    );

    populate_strip(
        &mut s,
        indices,
        1,
        cell_mesh_info,
        mesh_info,
        sample_info,
        shape_surface,
    );

    face.get_strips_mut()[strip_index] = s.clone();
}

/// 计算strip的一条边的顶点信息
pub fn populate_strip(
    strip: &mut Strip,
    indices: &[Vector3<usize>; 4],
    edge_index: usize,
    cell_mesh_info: &mut CellMeshInfo,
    mesh_info: &mut CMSMeshInfo,
    sample_info: &SampleInfo,
    shape_surface: Res<ShapeSurface>,
) {
    let edge = strip.get_edge(edge_index);
    assert!(edge.is_some());

    let vertex0 = VERTEX_MAP[edge.unwrap() as usize][0];
    let vertex1 = VERTEX_MAP[edge.unwrap() as usize][1];

    let vertex_coord0 = indices[vertex0 as usize];
    let vertex_coord1 = indices[vertex1 as usize];

    let mut vertex_range = Range::default();

    let edge_dir = get_edges_betwixt(&mut vertex_range, vertex_coord0, vertex_coord1);
    let edge_dir_index = edge_dir as usize;

    let sign_change_dir_coord = exact_sign_change_index(
        vertex_range.clone(),
        edge_dir,
        vertex_coord0,
        vertex_coord1,
        sample_info,
    );
    assert!(vertex_range.contains(&sign_change_dir_coord));

    let mut crossing_index_0 = vertex_coord0;
    let mut crossing_index_1 = vertex_coord1;

    crossing_index_0[edge_dir_index] = sign_change_dir_coord;
    crossing_index_1[edge_dir_index] = sign_change_dir_coord + 1;

    assert!(
        sample_info.sample_data.get_value(
            crossing_index_0.x,
            crossing_index_0.y,
            crossing_index_0.z
        ) * sample_info.sample_data.get_value(
            crossing_index_1.x,
            crossing_index_1.y,
            crossing_index_1.z
        ) <= 0.0
    );

    let mut dupli = false;

    let value_0 = mesh_info.vertex_index_data.get_value(
        crossing_index_0.x,
        crossing_index_0.y,
        crossing_index_0.z,
    );
    if value_0.is_empty() == false {
        if value_0
            .get_vertex_index()
            .get(edge_dir_index)
            .unwrap()
            .is_some()
        {
            strip.set_vertex_index(
                edge_index,
                value_0
                    .get_vertex_index()
                    .get(edge_dir_index)
                    .unwrap()
                    .unwrap(),
            );
            strip.set_crossing_left_coord(edge_index, crossing_index_0);
            strip.set_edge_dir(edge_index, Some(edge_dir));
            dupli = true;
        }
    }

    if dupli == false {
        make_vertex(
            strip,
            edge_dir,
            crossing_index_0,
            crossing_index_1,
            edge_index,
            cell_mesh_info,
            shape_surface,
        );
    }
}

/// 获取边的两个顶点的方向以及距离。
pub fn get_edges_betwixt(
    range: &mut Range<usize>,
    vertex_coord0: Vector3<usize>,
    vertex_coord1: Vector3<usize>,
) -> Direction {
    let mut direction = None;

    let diff = (vertex_coord0.cast::<i32>() - vertex_coord1.cast::<i32>()).abs();

    if diff.x > 0 {
        range.start = vertex_coord0.x.min(vertex_coord1.x) as usize;
        range.end = vertex_coord0.x.max(vertex_coord1.x) as usize;
        direction = Some(EdgeDirection::XAxis);
    } else if diff.y > 0 {
        range.start = vertex_coord0.y.min(vertex_coord1.y) as usize;
        range.end = vertex_coord0.y.max(vertex_coord1.y) as usize;
        direction = Some(EdgeDirection::YAxis);
    } else if diff.z > 0 {
        range.start = vertex_coord0.z.min(vertex_coord1.z) as usize;
        range.end = vertex_coord0.z.max(vertex_coord1.z) as usize;
        direction = Some(EdgeDirection::ZAxis);
    }

    assert!(direction.is_some());

    return direction.unwrap();
}

/// 检测是否有精确的符号变化。
/// 返回值为符号变化的前一个索引。
pub fn exact_sign_change_index(
    vertex_range: Range<usize>,
    edge_dir: Direction,
    vertex_coord0: Vector3<usize>,
    vertex_coord1: Vector3<usize>,
    sample_info: &SampleInfo,
) -> usize {
    let mut start_vertex_coord = Vector3::new(usize::MAX, usize::MAX, usize::MAX);

    if vertex_coord0[edge_dir as usize] == vertex_range.start {
        start_vertex_coord = vertex_coord0;
    } else if vertex_coord1[edge_dir as usize] == vertex_range.start {
        start_vertex_coord = vertex_coord1;
    }

    // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。
    if vertex_range.end - vertex_range.start == 1 {
        let this_value = sample_info.sample_data.get_value(
            start_vertex_coord.x,
            start_vertex_coord.y,
            start_vertex_coord.z,
        );
        let mut end_vertex_coord = start_vertex_coord;
        end_vertex_coord[edge_dir as usize] = start_vertex_coord[edge_dir as usize] + 1;
        let next_value = sample_info.sample_data.get_value(
            end_vertex_coord.x,
            end_vertex_coord.y,
            end_vertex_coord.z,
        );
        // info!(
        //     "this value {}, next_value: {}, start {}, end {}",
        //     this_value, next_value, start_vertex_coord, end_vertex_coord
        // );
        assert!(this_value * next_value <= 0.0);
        return start_vertex_coord[edge_dir as usize];
    }

    let mut indexer = start_vertex_coord;

    // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。
    for i in vertex_range.clone() {
        indexer[edge_dir as usize] = i;

        let this_value = sample_info
            .sample_data
            .get_value(indexer.x, indexer.y, indexer.z);

        indexer[edge_dir as usize] = i + 1;
        let next_value = sample_info
            .sample_data
            .get_value(indexer.x, indexer.y, indexer.z);

        if this_value * next_value <= 0.0 {
            return i;
        }
    }

    // for i in vertex_range {
    //     indexer[edge_dir as usize] = i;
    //
    //     let this_value = self.sample_data.get_value(indexer.x, indexer.y, indexer.z);
    //
    //     indexer[edge_dir as usize] = i + 1;
    //     let next_value = self.sample_data.get_value(indexer.x, indexer.y, indexer.z);
    //
    //     info!("this value: {} next value {}", this_value, next_value);
    //     if this_value * next_value <= 0.0 {
    //         return i;
    //     }
    // }
    // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。此处代码不会执行。
    assert!(false);

    return usize::MAX;
}

pub fn make_vertex(
    strip: &mut Strip,
    edge_dir: Direction,
    crossing_index_0: Vector3<usize>,
    crossing_index_1: Vector3<usize>,
    edge_index: usize,
    cell_mesh_info: &mut CellMeshInfo,
    mesh_info: &mut CMSMeshInfo,
    sample_info: &SampleInfo,
    shape_surface: Res<ShapeSurface>,
) {
    let pos0 =
        sample_info
            .sample_data
            .get_pos(crossing_index_0.x, crossing_index_0.y, crossing_index_0.z);
    let value0 = sample_info.sample_data.get_value(
        crossing_index_0.x,
        crossing_index_0.y,
        crossing_index_0.z,
    );
    let point0 = Point::new_with_position_and_value(&pos0, value0);

    let pos1 =
        sample_info
            .sample_data
            .get_pos(crossing_index_1.x, crossing_index_1.y, crossing_index_1.z);
    let value1 = sample_info.sample_data.get_value(
        crossing_index_1.x,
        crossing_index_1.y,
        crossing_index_1.z,
    );
    let point1 = Point::new_with_position_and_value(&pos1, value1);

    let crossing_point = find_crossing_point(2, &point0, &point1, shape_surface);
    let mut gradient = Vector3::new(0.0, 0.0, 0.0);
    find_gradient(
        &mut gradient,
        sample_info.offsets,
        &crossing_point,
        shape_surface,
    );

    let vert = Vertex::new_with_position_and_normals(&crossing_point, &gradient);
    mesh_info.vertices.push(vert);

    strip.set_vertex_index(edge_index, mesh_info.vertices.len() - 1);
    strip.set_crossing_left_coord(edge_index, crossing_index_0);
    strip.set_edge_dir(edge_index, Some(edge_dir));

    let mut e = mesh_info.vertex_index_data.get_value(
        crossing_index_0.x,
        crossing_index_0.y,
        crossing_index_0.z,
    );

    assert!(e
        .get_vertex_index()
        .get(edge_dir as usize)
        .unwrap()
        .is_none());
    e.set_dir_vertex_index(edge_dir, mesh_info.vertices.len() - 1);
    mesh_info.vertex_index_data.set_value(
        crossing_index_0.x,
        crossing_index_0.y,
        crossing_index_0.z,
        e.clone(),
    );
}

/// @param quality iter count
pub fn find_crossing_point(
    quality: usize,
    point0: &Point,
    point1: &Point,
    shape_surface: Res<ShapeSurface>,
) -> Vector3<f32> {
    let iso_value = shape_surface.get_iso_level();

    let p0 = point0.get_position();
    let v0 = point0.get_value();

    let p1 = point1.get_position();
    let v1 = point1.get_value();

    let alpha = (iso_value - v0) / (v1 - v0);
    let mut pos = p0 + (p1 - p0) * alpha;
    let val = shape_surface.get_value(pos.x, pos.y, pos.z);

    let point = Point::new_with_position_and_value(&pos, val);

    // 误差足够小，或者迭代次数足够多，就认为找到了交点。
    if (iso_value - val).abs() < f32::EPSILON || quality == 0 {
        return pos;
    } else {
        if val < 0.0 {
            if v0 > 0.0 {
                pos = find_crossing_point(quality - 1, &point, point0, shape_surface);
            } else if v1 > 0.0 {
                pos = find_crossing_point(quality - 1, &point, point1, shape_surface);
            }
        } else {
            if v0 < 0.0 {
                pos = find_crossing_point(quality - 1, point0, &point, shape_surface);
            } else if v1 < 0.0 {
                pos = find_crossing_point(quality - 1, point1, &point, shape_surface);
            }
        }
    }

    return pos;
}

pub fn find_gradient(
    normal: &mut Vector3<f32>,
    offset: Vector3<f32>,
    position: &Vector3<f32>,
    shape_surface: Res<ShapeSurface>,
) {
    let val = shape_surface.get_value(position.x, position.y, position.z);
    let dx = shape_surface.get_value(position.x + offset.x, position.y, position.z);

    let dy = shape_surface.get_value(position.x, position.y + offset.y, position.z);

    let dz = shape_surface.get_value(position.x, position.y, position.z + offset.z);

    *normal = Vector3::new(dx - val, dy - val, dz - val);
}

/// 计算面的Twin的Strip的起点和重点，以及所经过的顶点。
/// todo: 如果twin是由多个leaf Cell的面组成的，会重复吧，需要添加检测
pub fn edit_transitional_face(
    cells: Query<&Cell, &Faces>,
    octree_query: Query<(&Octree, &OctreeCellAddress)>,
) {
    info!("edit_transitional_face");

    for (octree, addresses) in octree_query.iter() {
        for cell_entity in octree.transit_face_celss {
            if let Ok(cell, faces) = cells.get(cell_entity) {
                for face_index in FaceIndex::iter() {
                    let face = faces.get_face(face_index);
                    assert!(face.get_face_type() == &FaceType::TransitFace);

                    let (twin_address, twin_face_index) = cell.get_twin_face_address(face_index);
                    let twin_cell_entity = addresses.get(twin_address);

                    let Ok(twin_cell, twin_faces) = cells.get(twin_cell_entity);

                    let mut all_strips = Vec::new();
                    traverse_face(
                        cells,
                        addresses,
                        twin_cell,
                        twin_faces.get_face(face_index),
                        &mut all_strips,
                    );

                    if all_strips.len() == 0 {
                        continue;
                    }

                    let mut transit_segs = Vec::new();

                    loop {
                        let mut vertex_indices = Vec::new();

                        if let Some(data) = all_strips[0].get_vertex_index(0) {
                            vertex_indices.push(data);
                        }
                        if let Some(data) = all_strips[0].get_vertex_index(1) {
                            vertex_indices.push(data);
                        }

                        let mut long_strip = all_strips[0].clone();

                        all_strips.remove(0);

                        let mut added_in_iteration;

                        loop {
                            added_in_iteration = 0;

                            all_strips.retain(|strip| {
                                if vertex_indices.last() == strip.get_vertex_index(0).as_ref() {
                                    if let Some(data) = strip.get_vertex_index(1) {
                                        vertex_indices.push(data);
                                        long_strip.change_back(strip, 1);
                                        added_in_iteration += 1;
                                    } else {
                                        // info!("Some(data) != strip.get_vertex_index(1)");
                                    }
                                } else if vertex_indices.last()
                                    == strip.get_vertex_index(1).as_ref()
                                {
                                    if let Some(data) = strip.get_vertex_index(0) {
                                        vertex_indices.push(data);
                                        long_strip.change_back(strip, 0);
                                        added_in_iteration += 1;
                                    } else {
                                        // info!("Some(data) != strip.get_vertex_index(0)");
                                    }
                                } else {
                                    // info!("all_strips.retain first false");
                                    // info!(
                                    //     "strip: {:?} vertex_indices: {:?}",
                                    //     strip, vertex_indices
                                    // );
                                    return true;
                                }

                                if vertex_indices.first() == vertex_indices.last() {
                                    vertex_indices.remove(0);
                                    long_strip.set_loop(true);
                                } else {
                                    // info!(
                                    //     "!= vertex index len {} vertex index {:?} added_in_iteration {}, long_strip is loop: {}",
                                    //     vertex_indices.len(),
                                    //     vertex_indices,
                                    //     added_in_iteration,
                                    //     long_strip.get_loop()
                                    // );
                                }

                                return false;
                            });

                            // info!("all_strips: num: {}", all_strips.len());

                            if all_strips.len() <= 0
                                || added_in_iteration <= 0
                                || long_strip.get_loop()
                            {
                                // info!("all_strips.retain first break");
                                break;
                            } else {
                                // info!("first all_strips: len {} added_in_iteration {}, long_strip is loop: {}", all_strips.len(), added_in_iteration, long_strip.get_loop());
                            }
                        }

                        if long_strip.get_loop() == false && all_strips.len() > 0 {
                            loop {
                                added_in_iteration = 0;

                                all_strips.retain(|strip| {
                                    if vertex_indices.first() == strip.get_vertex_index(0).as_ref()
                                    {
                                        if let Some(data) = strip.get_vertex_index(1) {
                                            vertex_indices.insert(0, data);
                                            long_strip.change_front(strip, 1);
                                            added_in_iteration += 1;
                                        }
                                    } else if vertex_indices.first()
                                        == strip.get_vertex_index(1).as_ref()
                                    {
                                        if let Some(data) = strip.get_vertex_index(0) {
                                            vertex_indices.insert(0, data);
                                            long_strip.change_front(strip, 0);
                                            added_in_iteration += 1;
                                        }
                                    } else {
                                        // info!("all_strips.retain second false");
                                        return true;
                                    }

                                    if vertex_indices.first() == vertex_indices.last() {
                                        vertex_indices.remove(0);
                                        long_strip.set_loop(true);
                                    }
                                    return false;
                                });

                                if all_strips.len() <= 0
                                    || added_in_iteration <= 0
                                    || long_strip.get_loop()
                                {
                                    break;
                                } else {
                                    // info!("seconds all_strips: len {} added_in_iteration {}, long_strip is loop: {}", all_strips.len(), added_in_iteration, long_strip.get_loop());
                                }
                            }
                        }

                        twin_faces
                            .get_face(face_index)
                            .get_strips_mut()
                            .push(long_strip.clone());

                        transit_segs.push(vertex_indices);

                        // info!("all_strips len: {}", all_strips.len());
                        if all_strips.len() == 0 {
                            break;
                        } else {
                            // info!("three all_strips: len {} added_in_iteration {}, long_strip is loop: {}", all_strips.len(), added_in_iteration, long_strip.get_loop());
                        }
                    }

                    if transit_segs.len() != 0 {
                        twin_faces
                            .get_face(face_index)
                            .set_transit_segs(transit_segs);
                    }

                    all_strips.clear();
                }
            }
        }
    }
}

pub fn traverse_face(
    cell_query: Query<(&Cell, &Faces)>,
    cell_addresses: &OctreeCellAddress,
    cell: &Cell,
    face: &Face,
    strips: &mut Vec<Strip>,
) {
    match face.get_face_type() {
        FaceType::BranchFace => {
            let face_index = face.get_face_index();
            let sub_cell_indices = FACE_2_SUBCELL[face_index as usize];

            for sub_cell_index in sub_cell_indices.iter() {
                let child_address = cell.get_address().get_child_address(sub_cell_index);
                let entity = cell_addresses.cell_addresses.get(&child_address);
                let (cell, faces) = cell_query.get(entity);
                traverse_face(
                    cell_query,
                    cell_addresses,
                    cell,
                    faces.get(face_index),
                    strips,
                );
            }
        }
        FaceType::LeafFace => {
            for strip in face.get_strips().iter() {
                if strip.get_vertex_index(0).is_none() {
                    continue;
                }
                strips.push(strip.clone());
            }
        }
        FaceType::TransitFace => assert!(false),
    }
}

pub fn trace_comonent(
    cells: Query<(&Cell, &mut CellMeshInfo)>,
    query: Query<(&Octree, &OctreeCellAddress)>,
) {
    info!("trace_comonent");

    for (octree, mut cell_addresses) in query.iter_mut() {
        for cell_entity in octree.leaf_cells.iter() {
            let Ok(cell, cell_mesh_info) = cells.get(cell_entity) else {
                continue;
            };

            let mut cell_strips = Vec::new();
            let mut transit_segs = Vec::new();
            let mut components = Vec::new();

            // 获取一个cell的所有strip
            collect_strips(
                cells,
                cell_addresses,
                cell,
                &mut cell_strips,
                &mut transit_segs,
            );

            // todo: transit segs number is not correct

            // 循环是为了建立多个Component
            loop {
                if cell_strips.len() == 0 {
                    break;
                }

                link_strips(&mut components, &mut cell_strips, &mut transit_segs);

                cell_mesh_info.components.push(components.clone());

                components.clear();
            }
        }
    }
}
pub fn collect_strips(
    cells: Query<&Cell>,
    cell_addresses: &OctreeCellAddress,
    cell: &Cell,
    cell_strips: &mut Vec<Strip>,
    transit_segs: &mut Vec<Vec<usize>>,
) {
    for face_index in FaceIndex::iter() {
        let face = cell.get_face(face_index);
        match face.get_face_type() {
            FaceType::BranchFace => {
                assert!(false);
            }
            FaceType::LeafFace => {
                for strip in face.get_strips().iter() {
                    if strip.get_vertex_index(0).is_some() {
                        cell_strips.push(strip.clone());
                    }
                }
            }
            FaceType::TransitFace => {
                for strip in face.get_strips().iter() {
                    if strip.get_vertex_index(0).is_some() {
                        cell_strips.push(strip.clone());
                    }
                }

                let (twin_address, twin_face_index) = cell.get_twin_face_address(face_index);
                let cell_entity = cell_addresses.get(twin_address);
                let (cell, faces) = cells.get(cell_entity);

                let twin = faces.get_face(twin_face_index);
                for (i, strip) in twin.get_strips().iter().enumerate() {
                    strip.get_vertex_index(0).map(|data| {
                        transit_segs.push(twin.get_transit_segs()[i].clone());
                        cell_strips.push(strip.clone());
                    });
                }
            }
        }
    }

    assert!(cell_strips.len() > 0);
}

pub fn link_strips(
    components: &mut Vec<usize>,
    cell_strips: &mut Vec<Strip>,
    transit_segs: &mut Vec<Vec<usize>>,
) {
    assert!(components.len() == 0);
    assert!(cell_strips[0].get_vertex_index(0).is_some());

    let mut added_in_iteration;
    let mut backwards = false;

    components.push(cell_strips[0].get_vertex_index(0).unwrap());

    loop {
        added_in_iteration = 0;

        cell_strips.retain(|strip| {
            assert!(strip.get_vertex_index(0).is_some() && strip.get_vertex_index(1).is_some());

            let s_data0 = strip.get_vertex_index(0);
            let s_data1 = strip.get_vertex_index(1);

            match components.last() {
                Some(v) if Some(*v) == s_data0 => {
                    backwards = false;
                    let mut transit = false;

                    if transit_segs.len() > 0 {
                        insert_data_from_twin(
                            components,
                            &transit_segs,
                            &strip,
                            &mut transit,
                            &mut added_in_iteration,
                            &backwards,
                        );
                        // if debug {
                        //     info!("component transit: {:?}", components);
                        // }
                    }

                    if transit == false {
                        if let Some(data) = s_data1 {
                            components.push(data);
                            added_in_iteration += 1;
                            // if debug {
                            //     info!("component non transit: {:?}", components);
                            // }
                        }
                    }
                }
                Some(v) if Some(*v) == s_data1 => {
                    backwards = true;
                    let mut transit = false;

                    if transit_segs.len() > 0 {
                        insert_data_from_twin(
                            components,
                            &transit_segs,
                            &strip,
                            &mut transit,
                            &mut added_in_iteration,
                            &backwards,
                        );
                        // if debug {
                        //     info!("component transit 2: {:?}", components);
                        // }
                    }

                    if transit == false {
                        if let Some(data) = s_data0 {
                            components.push(data);
                            added_in_iteration += 1;
                            // if debug {
                            //     info!("component non transit 2: {:?}", components);
                            // }
                        }
                    }
                }
                _ => {
                    // if debug {
                    //     info!("component not add");
                    // }
                    return true;
                }
            }
            return false;
        });

        if components.first() == components.last() {
            components.remove(0);
        }

        // if components.len() > 1 && components.first() == components.get(1) {
        //     components.remove(0);
        // }
        //
        if added_in_iteration <= 0 {
            break;
        }
    }

    loop {
        added_in_iteration = 0;

        cell_strips.retain(|strip| {
            assert!(strip.get_vertex_index(0).is_some() && strip.get_vertex_index(1).is_some());

            let s_data0 = strip.get_vertex_index(0);
            let s_data1 = strip.get_vertex_index(1);

            match components.first() {
                Some(v) if Some(*v) == s_data0 => {
                    backwards = false;
                    let mut transit = false;

                    if transit_segs.len() > 0 {
                        insert_data_from_twin(
                            components,
                            &transit_segs,
                            &strip,
                            &mut transit,
                            &mut added_in_iteration,
                            &backwards,
                        );
                        // if debug {
                        //     info!("component transit 3: {:?}", components);
                        // }
                    }

                    if transit == false {
                        if let Some(data) = s_data1 {
                            components.insert(0, data);
                            added_in_iteration += 1;
                            // if debug {
                            //     info!("component non transit 3: {:?}", components);
                            // }
                        }
                    }
                }
                Some(v) if Some(*v) == s_data1 => {
                    backwards = true;
                    let mut transit = false;

                    if transit_segs.len() > 0 {
                        insert_data_from_twin(
                            components,
                            &transit_segs,
                            &strip,
                            &mut transit,
                            &mut added_in_iteration,
                            &backwards,
                        );
                        // if debug {
                        //     info!("component transit 4: {:?}", components);
                        // }
                    }

                    if transit == false {
                        if let Some(data) = s_data0 {
                            components.insert(0, data);
                            added_in_iteration += 1;
                            // if debug {
                            //     info!("component non transit 4: {:?}", components);
                            // }
                        }
                    }
                }
                _ => {
                    // if debug {
                    //     info!("component 2 not add");
                    // }
                    return true;
                }
            }
            return false;
        });

        if components.first() == components.last() {
            components.remove(0);
        }

        // if components.len() > 1 && components.first() == components.get(1) {
        //     components.remove(0);
        // }

        if added_in_iteration <= 0 {
            break;
        }
    }

    assert!(components.first() != components.last());
    assert!(components.len() >= 3);
    // if components.len() < 3 {
    //     components.clear();
    // }
}

fn insert_data_from_twin(
    components: &mut Vec<usize>,
    transit_segs: &Vec<Vec<usize>>,
    strip: &Strip,
    transit: &mut bool,
    added_in_iteration: &mut i32,
    backwards: &bool,
) {
    for seg in transit_segs.iter() {
        if compare_strip_to_seg(strip, seg) {
            // if debug {
            //     info!(
            //         "success component cell_strips: {:?} {:?}",
            //         strip.get_vertex_index(0),
            //         strip.get_vertex_index(1)
            //     );
            //     info!("success component seg: {:?}", seg,);
            // }
            if *backwards {
                for i in (0..seg.len()).rev() {
                    components.push(seg[i] as usize);
                }
            } else {
                for i in 0..seg.len() {
                    components.push(seg[i] as usize);
                }
            }
            // transit_segs.remove(i);
            *added_in_iteration += 1;
            *transit = true;
            break;
        } else {
            // if debug {
            //     info!(
            //         "fail component cell_strips: {:?} {:?}",
            //         strip.get_vertex_index(0),
            //         strip.get_vertex_index(1)
            //     );
            //     info!("fail component seg: {:?}", seg,);
            // }
        }
    }
}

fn compare_strip_to_seg(strip: &Strip, seg: &Vec<usize>) -> bool {
    let s0 = strip.get_vertex_index(0);
    let s1 = strip.get_vertex_index(1);

    (seg.first() == s0.as_ref() && seg.last() == s1.as_ref())
        || (seg.first() == s1.as_ref() && seg.last() == s0.as_ref())
}
