use std::{
    ops::{Not, Range},
    sync::{Arc, RwLock},
};

use bevy::{prelude::*, utils::HashMap};

use strum::{EnumCount, IntoEnumIterator};

use crate::terrain::isosurface::{
    cms::build::{
        address::VoxelAddress,
        cell::Cell,
        face::{Face, FaceType},
        octree::Octree,
        point::Point,
        strip::Strip,
        tables::{
            EdgeDirection, Face2DEdge, Face2DVertex, FaceIndex, EDGE_MAP, FACE_2_SUBCELL,
            FACE_VERTEX, VERTEX_MAP,
        },
    },
    cms::{bundle::CMSVertexIndexInfo, sample::surface_sampler::SurfaceSampler},
    mesh::mesh_cache::MeshCache,
    surface::shape_surface::ShapeSurface,
};

impl Octree {
    /// 生成每个面的连线，以及边的顶点的位置信息。
    pub fn generate_segments(
        &mut self,
        shape_surface: Arc<RwLock<ShapeSurface>>,
        surface_sampler: Arc<RwLock<SurfaceSampler>>,
        mesh_cache: Arc<RwLock<MeshCache>>,
        vertex_info: Arc<RwLock<CMSVertexIndexInfo>>,
    ) {
        let mut surface_sampler = surface_sampler.write().unwrap();
        let mut mesh_cache = mesh_cache.write().unwrap();
        let mut vertex_info = vertex_info.write().unwrap();
        let mut shape_surface = shape_surface.write().unwrap();

        info_span!("generate_segments");
        info!(
            "generate_segments: octree leaf cell num: {}",
            self.leaf_cells.len()
        );

        for address in self.leaf_cells.iter() {
            if let Some(cell) = self.cell_addresses.get_mut(address) {
                // let Ok(cell_mesh_info) = cell_mesh_info.get_mut(*entity) else {
                //     debug_assert!(false);
                //     continue;
                // };
                //
                let mut indices = [UVec3::ZERO; Face2DVertex::COUNT];
                let mut valid_strips = 0;
                for face_index in FaceIndex::iter() {
                    for (i, face_vertex) in Face2DVertex::iter().enumerate() {
                        let vertex_pos = FACE_VERTEX[face_index as usize][face_vertex as usize];
                        indices[i] = cell.get_corner_sample_index()[vertex_pos as usize];
                        debug!("vertex_pos:{i} {:?}, indices: {}", vertex_pos, indices[i]);
                    }

                    debug!("face index {:?} start", face_index);
                    let face = cell.faces.get_face_mut(face_index);
                    face.get_strips_mut().resize(2, Strip::default());
                    Self::make_face_segments(
                        &indices,
                        face,
                        &mut mesh_cache,
                        &mut vertex_info,
                        &mut surface_sampler,
                        &mut shape_surface,
                    );
                    if face.get_strips()[0].get_vertex_index(0).is_some() {
                        valid_strips += 1;
                    }
                }

                for face_index in FaceIndex::iter() {
                    let face = cell.faces.get_face(face_index);
                    for strip in face.get_strips().iter() {
                        for vertex_index in strip.get_vertex().iter() {
                            debug!("entity: {:?}: vertex_index: {:?}", address, vertex_index);
                        }
                    }
                }

                debug_assert!(valid_strips > 0);
            }
        }
    }

    // make segments in a face.
    fn make_face_segments(
        indices: &[UVec3; 4],
        face: &mut Face,
        mesh_info: &mut MeshCache,
        vertex_info: &mut CMSVertexIndexInfo,
        sample_info: &mut SurfaceSampler,
        shape_surface: &mut ShapeSurface,
    ) {
        let edges = (0..4).fold(0, |acc, i| {
            debug!(
                "index: {}, value: {}",
                indices[i],
                sample_info.get_value_from_vertex_address(indices[i], shape_surface)
            );
            acc | if sample_info.get_value_from_vertex_address(indices[i], shape_surface) < 0.0 {
                1 << i
            } else {
                0
            }
        });

        debug!("make_face_segments edges {}", edges);

        let e0a = EDGE_MAP[edges as usize][0][0];
        let e0b = EDGE_MAP[edges as usize][0][1];

        if e0a.is_some() {
            Self::make_strip(
                e0a,
                e0b,
                indices,
                face,
                0,
                mesh_info,
                vertex_info,
                sample_info,
                shape_surface,
            );
        }

        let e1a = EDGE_MAP[edges as usize][1][0];
        let e1b = EDGE_MAP[edges as usize][1][1];

        if e1a.is_some() {
            Self::make_strip(
                e1a,
                e1b,
                indices,
                face,
                1,
                mesh_info,
                vertex_info,
                sample_info,
                shape_surface,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn make_strip(
        edge0: Option<Face2DEdge>,
        edge1: Option<Face2DEdge>,
        indices: &[UVec3; 4],
        face: &mut Face,
        strip_index: usize,
        mesh_info: &mut MeshCache,
        vertex_info: &mut CMSVertexIndexInfo,
        sample_info: &mut SurfaceSampler,
        shape_surface: &mut ShapeSurface,
    ) {
        debug!("make strip {}: {:?} {:?}", strip_index, edge0, edge1);
        debug_assert!(edge0.is_some() && edge1.is_some());

        let mut s = Strip::new(edge0, edge1);

        Self::populate_strip(
            &mut s,
            indices,
            0,
            mesh_info,
            vertex_info,
            sample_info,
            shape_surface,
        );

        Self::populate_strip(
            &mut s,
            indices,
            1,
            mesh_info,
            vertex_info,
            sample_info,
            shape_surface,
        );

        if strip_index == 1 {
            debug_assert!(face.get_strips_mut()[0] != s);
        }
        face.get_strips_mut()[strip_index] = s.clone();
    }

    /// 计算strip的一条边的顶点信息
    fn populate_strip(
        strip: &mut Strip,
        indices: &[UVec3; 4],
        edge_index: usize,
        mesh_info: &mut MeshCache,
        vertex_info: &mut CMSVertexIndexInfo,
        sample_info: &mut SurfaceSampler,
        shape_surface: &mut ShapeSurface,
    ) {
        debug!("populate_strip strip edge index {}", edge_index);
        let edge = strip.get_edge(edge_index);
        debug_assert!(edge.is_some());

        let vertex0 = VERTEX_MAP[edge.unwrap() as usize][0];
        let vertex1 = VERTEX_MAP[edge.unwrap() as usize][1];

        // todo: 因为是从叶子cell的面四边形的四个顶点开始，所以这里的顶点索引是固定的，后面可以优化
        let vertex_coord0 = indices[vertex0 as usize];
        let vertex_coord1 = indices[vertex1 as usize];

        debug!(
            "vertex_coord0: {}, vertex_coord1: {}",
            vertex_coord0, vertex_coord1
        );

        let mut vertex_range = Range::default();

        let edge_dir = Self::get_edges_betwixt(&mut vertex_range, vertex_coord0, vertex_coord1);
        let edge_dir_index = edge_dir as usize;

        let sign_change_dir_coord = Self::exact_sign_change_index(
            vertex_range.clone(),
            edge_dir,
            vertex_coord0,
            vertex_coord1,
            sample_info,
            shape_surface,
        );
        debug_assert!(vertex_range.contains(&sign_change_dir_coord));

        let mut crossing_index_0 = vertex_coord0;
        let mut crossing_index_1 = vertex_coord1;

        crossing_index_0[edge_dir_index] = sign_change_dir_coord;
        crossing_index_1[edge_dir_index] = sign_change_dir_coord + 1;

        debug_assert!(
            sample_info.get_value_from_vertex_address(crossing_index_0, shape_surface)
                * sample_info.get_value_from_vertex_address(crossing_index_1, shape_surface)
                <= 0.0
        );

        let mut duplicate = false;

        debug!(
            "crossing_index_0: {}, edge_dir: {:?}",
            crossing_index_0, edge_dir
        );

        if let Some(value_0) = vertex_info.vertex_index_info.get(&crossing_index_0) {
            if value_0.get_dir_vertex_index(edge_dir).is_some() {
                debug!("vertex index: {:?}", value_0.get_dir_vertex_index(edge_dir));
                strip.set_vertex_index(edge_index, value_0.get_dir_vertex_index(edge_dir).unwrap());
                duplicate = true;
                debug!("make vertex duplicate");
            } else {
                debug!("crossing_index_0 get_dir_vertex_index is None");
            }
        } else {
            debug!("mesh_info.vertex_index_data.get(&crossing_index_0) is None");
        }

        if duplicate.not() {
            Self::make_vertex(
                strip,
                edge_dir,
                crossing_index_0,
                crossing_index_1,
                edge_index,
                mesh_info,
                sample_info,
                shape_surface,
                vertex_info,
            );
        }
    }

    /// 获取边的两个顶点的方向以及距离。
    fn get_edges_betwixt(
        range: &mut Range<u32>,
        vertex_coord0: UVec3,
        vertex_coord1: UVec3,
    ) -> EdgeDirection {
        let mut direction = None;

        let diff = (vertex_coord0.as_ivec3() - vertex_coord1.as_ivec3()).abs();

        if diff.x > 0 {
            range.start = vertex_coord0.x.min(vertex_coord1.x);
            range.end = vertex_coord0.x.max(vertex_coord1.x);
            direction = Some(EdgeDirection::XAxis);
        } else if diff.y > 0 {
            range.start = vertex_coord0.y.min(vertex_coord1.y);
            range.end = vertex_coord0.y.max(vertex_coord1.y);
            direction = Some(EdgeDirection::YAxis);
        } else if diff.z > 0 {
            range.start = vertex_coord0.z.min(vertex_coord1.z);
            range.end = vertex_coord0.z.max(vertex_coord1.z);
            direction = Some(EdgeDirection::ZAxis);
        }

        debug_assert!(direction.is_some());

        direction.unwrap()
    }

    /// 检测是否有精确的符号变化。
    /// 返回值为符号变化的前一个索引。
    fn exact_sign_change_index(
        vertex_range: Range<u32>,
        edge_dir: EdgeDirection,
        vertex_coord0: UVec3,
        vertex_coord1: UVec3,
        sample_info: &mut SurfaceSampler,
        shape_surface: &mut ShapeSurface,
    ) -> u32 {
        let mut start_vertex_coord = UVec3::splat(u32::MAX);

        if vertex_coord0[edge_dir as usize] == vertex_range.start {
            start_vertex_coord = vertex_coord0;
        } else if vertex_coord1[edge_dir as usize] == vertex_range.start {
            start_vertex_coord = vertex_coord1;
        }

        // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。
        if vertex_range.end - vertex_range.start == 1 {
            let this_value =
                sample_info.get_value_from_vertex_address(start_vertex_coord, shape_surface);
            let mut end_vertex_coord = start_vertex_coord;
            end_vertex_coord[edge_dir as usize] = start_vertex_coord[edge_dir as usize] + 1;
            let next_value =
                sample_info.get_value_from_vertex_address(end_vertex_coord, shape_surface);
            // debug!(
            //     "this value {}, next_value: {}, start {}, end {}",
            //     this_value, next_value, start_vertex_coord, end_vertex_coord
            // );
            debug_assert!(this_value * next_value <= 0.0);
            // debug_assert!(this_value * next_value != 0.0);
            return start_vertex_coord[edge_dir as usize];
        }

        let mut indexer = start_vertex_coord;

        // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。
        for i in vertex_range.clone() {
            indexer[edge_dir as usize] = i;

            let this_value = sample_info.get_value_from_vertex_address(indexer, shape_surface);

            indexer[edge_dir as usize] = i + 1;
            let next_value = sample_info.get_value_from_vertex_address(indexer, shape_surface);

            // debug_assert!(this_value * next_value != 0.0);
            if this_value * next_value <= 0.0 {
                return i;
            }
        }

        // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。此处代码不会执行。
        debug_assert!(false);

        u32::MAX
    }
    #[allow(clippy::too_many_arguments)]
    fn make_vertex(
        strip: &mut Strip,
        edge_dir: EdgeDirection,
        crossing_index_0: UVec3,
        crossing_index_1: UVec3,
        edge_index: usize,
        mesh_info: &mut MeshCache,
        sample_info: &mut SurfaceSampler,
        shape_surface: &mut ShapeSurface,
        vertex_info: &mut CMSVertexIndexInfo,
    ) {
        debug!("make vertex");
        let pos0 = sample_info.get_pos_from_vertex_address(crossing_index_0, shape_surface);
        let value0 = sample_info.get_value_from_vertex_address(crossing_index_0, shape_surface);
        let point0 = Point::new_with_position_and_value(pos0, value0);
        debug!("crossing_index_0:{} point0:{:?}", crossing_index_0, point0);

        let pos1 = sample_info.get_pos_from_vertex_address(crossing_index_1, shape_surface);
        let value1 = sample_info.get_value_from_vertex_address(crossing_index_1, shape_surface);
        let point1 = Point::new_with_position_and_value(pos1, value1);
        debug!("crossing_index_1:{} point1:{:?}", crossing_index_1, point1);

        let crossing_vertex_point_pos =
            Self::find_crossing_point_pos(0, &point0, &point1, sample_info, shape_surface);
        let mut gradient = Vec3::ZERO;
        Self::find_gradient(
            &mut gradient,
            crossing_vertex_point_pos,
            sample_info,
            shape_surface,
        );
        debug!(
            "crossing_point:{} gradient:{}",
            crossing_vertex_point_pos, gradient
        );

        mesh_info.positions.push(crossing_vertex_point_pos);
        mesh_info.normals.push(gradient);

        debug!("add vertex index: {:?}", mesh_info.positions.len() - 1);
        strip.set_vertex_index(edge_index, (mesh_info.positions.len() - 1) as u32);

        let vertex_index = vertex_info
            .vertex_index_info
            .entry(crossing_index_0)
            .or_default();
        debug_assert!(vertex_index.get_dir_vertex_index(edge_dir).is_none());
        vertex_index.set_dir_vertex_index(edge_dir, (mesh_info.positions.len() - 1) as u32);
    }

    /// @param quality iter count
    fn find_crossing_point_pos(
        quality: usize,
        point0: &Point,
        point1: &Point,
        sample_info: &SurfaceSampler,
        shape_surface: &ShapeSurface,
    ) -> Vec3 {
        let p0 = point0.get_position();
        let v0 = point0.get_value();

        let p1 = point1.get_position();
        let v1 = point1.get_value();

        let alpha_value = if v1 - v0 != 0.0 {
            (0.0 - v0) / (v1 - v0)
        } else {
            0.0
        };
        debug_assert!(alpha_value >= 0.0);
        let mut pos = *p0 + (*p1 - *p0) * alpha_value;
        let val = sample_info.get_value_from_pos(pos, shape_surface);

        let point = Point::new_with_position_and_value(pos, val);

        // 误差足够小，或者迭代次数足够多，就认为找到了交点。
        if val.abs() < f32::EPSILON || quality == 0 {
            return pos;
        } else if val < 0.0 {
            if v0 > 0.0 {
                pos = Self::find_crossing_point_pos(
                    quality - 1,
                    &point,
                    point0,
                    sample_info,
                    shape_surface,
                );
            } else if v1 > 0.0 {
                pos = Self::find_crossing_point_pos(
                    quality - 1,
                    &point,
                    point1,
                    sample_info,
                    shape_surface,
                );
            }
        } else if v0 < 0.0 {
            pos = Self::find_crossing_point_pos(
                quality - 1,
                point0,
                &point,
                sample_info,
                shape_surface,
            );
        } else if v1 < 0.0 {
            pos = Self::find_crossing_point_pos(
                quality - 1,
                point1,
                &point,
                sample_info,
                shape_surface,
            );
        }

        pos
    }

    fn find_gradient(
        normal: &mut Vec3,
        vertex_point_pos: Vec3,
        sample_info: &SurfaceSampler,
        shape_surface: &ShapeSurface,
    ) {
        let val = sample_info.get_value_from_pos(vertex_point_pos, shape_surface);
        let offset = sample_info.voxel_size * 0.5;
        let dx = sample_info.get_value_from_pos(
            vertex_point_pos + Vec3::new(offset.x, 0.0, 0.0),
            shape_surface,
        );

        let dy = sample_info.get_value_from_pos(
            vertex_point_pos + Vec3::new(0.0, offset.y, 0.0),
            shape_surface,
        );

        let dz = sample_info.get_value_from_pos(
            vertex_point_pos + Vec3::new(0.0, 0.0, offset.z),
            shape_surface,
        );

        *normal = Vec3::new(dx - val, dy - val, dz - val).normalize();
    }

    /// 计算面的Twin的Strip的起点和重点，以及所经过的顶点。
    /// todo: 如果twin是由多个leaf Cell的面组成的，会重复吧，需要添加检测
    pub fn edit_transitional_face(&mut self) {
        info_span!("edit_transitional_face");
        info!(
            "edit_transitional_face: octree.transit_face_cells.len(): {}",
            self.transit_face_cells.len()
        );

        // todo: cache transit face indices....
        for transit_cell_address in self.transit_face_cells.iter() {
            let mut all_strips = [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ];
            let mut all_twin_cell_address = [
                VoxelAddress::default(),
                VoxelAddress::default(),
                VoxelAddress::default(),
                VoxelAddress::default(),
                VoxelAddress::default(),
                VoxelAddress::default(),
            ];
            let mut all_twin_cell_face_index = [
                Option::<FaceIndex>::None,
                Option::<FaceIndex>::None,
                Option::<FaceIndex>::None,
                Option::<FaceIndex>::None,
                Option::<FaceIndex>::None,
                Option::<FaceIndex>::None,
            ];
            if let Some(cell) = self.cell_addresses.get(transit_cell_address) {
                for (i, face_index) in FaceIndex::iter().enumerate() {
                    let face = cell.faces.get_face(face_index);
                    if face.get_face_type() == &FaceType::TransitFace {
                        let (twin_address, twin_face_index) =
                            cell.get_twin_face_address(face_index);
                        if let Some(twin_cell) = self.cell_addresses.get(&twin_address) {
                            all_twin_cell_address[i] = twin_address;
                            all_twin_cell_face_index[i] = Some(twin_face_index);

                            let all_strip = &mut all_strips[i];
                            Self::traverse_face(
                                &self.cell_addresses,
                                all_twin_cell_address[i],
                                twin_face_index,
                                all_strip,
                            );

                            debug!(
                                "twin_entity: {:?}, to traverse_face, all strip: {:?}",
                                twin_cell, all_strip
                            );
                        } else {
                            debug!("get addresses.cell_addresses is none");
                        }
                    }
                }
            }

            for i in 0..6 {
                debug!(
                    "edit_transition_face: entity: {:?}, face_index: {:?}, all_strip: {:?}",
                    all_twin_cell_address[i], all_twin_cell_face_index[i], all_strips[i]
                );
            }

            for (index, twin_cell_address) in all_twin_cell_address.iter().enumerate() {
                if let Some(twin_cell) = self.cell_addresses.get_mut(twin_cell_address) {
                    let all_strip = &mut all_strips[index];
                    let twin_face_index = all_twin_cell_face_index[index].unwrap();

                    debug!(
                        "all twin_cell_entity {:?} face index: {:?} strip: {:?}",
                        twin_cell_address, index, all_strip
                    );

                    if all_strip.is_empty() {
                        debug!("all_strip.len() == 0 and continue");
                        continue;
                    }

                    let mut transit_segments = Vec::new();

                    loop {
                        let mut vertex_indices = Vec::new();

                        if let Some(data) = all_strip[0].get_vertex_index(0) {
                            vertex_indices.push(data);
                        }
                        if let Some(data) = all_strip[0].get_vertex_index(1) {
                            vertex_indices.push(data);
                        }

                        // 记录两个端点
                        let mut long_strip = all_strip[0].clone();

                        all_strip.remove(0);

                        let mut added_in_iteration;

                        loop {
                            added_in_iteration = 0;

                            all_strip.retain(|strip| {
                                    if vertex_indices.last() == strip.get_vertex_index(0).as_ref() {
                                        if let Some(data) = strip.get_vertex_index(1) {
                                            vertex_indices.push(data);
                                            long_strip.change_back(strip, 1);
                                            added_in_iteration += 1;
                                        } else {
                                            debug!("Some(data) != strip.get_vertex_index(1)");
                                        }
                                    } else if vertex_indices.last()
                                        == strip.get_vertex_index(1).as_ref()
                                    {
                                        if let Some(data) = strip.get_vertex_index(0) {
                                            vertex_indices.push(data);
                                            long_strip.change_back(strip, 0);
                                            added_in_iteration += 1;
                                        } else {
                                            debug!("Some(data) != strip.get_vertex_index(0)");
                                        }
                                    } else {
                                        debug!("all_strip.retain first false");
                                        debug!(
                                            "strip: {:?} vertex_indices: {:?}",
                                            strip, vertex_indices
                                        );
                                        return true;
                                    }

                                    if vertex_indices.first() == vertex_indices.last() {
                                        vertex_indices.remove(0);
                                        long_strip.set_loop(true);
                                    } else {
                                        debug!(
                                            "!= vertex index len {} vertex index {:?} added_in_iteration {}, long_strip is loop: {}",
                                            vertex_indices.len(),
                                            vertex_indices,
                                            added_in_iteration,
                                            long_strip.get_loop()
                                        );
                                    }

                                     false
                                });

                            debug!("all_strip: num: {}", all_strip.len());

                            if all_strip.is_empty()
                                || added_in_iteration == 0
                                || long_strip.get_loop()
                            {
                                debug!("all_strip.retain first break");
                                break;
                            } else {
                                debug!("first all_strip: len {} added_in_iteration {}, long_strip is loop: {}", all_strip.len(), added_in_iteration, long_strip.get_loop());
                            }
                        }

                        if long_strip.get_loop().not() && all_strip.is_empty().not() {
                            loop {
                                added_in_iteration = 0;

                                all_strip.retain(|strip| {
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
                                        debug!("all_strip.retain second false");
                                        return true;
                                    }

                                    if vertex_indices.first() == vertex_indices.last() {
                                        vertex_indices.remove(0);
                                        long_strip.set_loop(true);
                                    }
                                    false
                                });

                                if all_strip.is_empty()
                                    || added_in_iteration <= 0
                                    || long_strip.get_loop()
                                {
                                    break;
                                } else {
                                    debug!("seconds all_strip: len {} added_in_iteration {}, long_strip is loop: {}", all_strip.len(), added_in_iteration, long_strip.get_loop());
                                }
                            }
                        }

                        debug!(
                            "twin_faces faceIndex: {:?} strips: {:?}, and long strip: {:?}",
                            twin_face_index,
                            twin_cell.faces.get_face_mut(twin_face_index).get_strips(),
                            long_strip
                        );
                        debug_assert!(
                            long_strip.get_vertex_index(0).is_some()
                                && twin_cell
                                    .faces
                                    .get_face_mut(twin_face_index)
                                    .get_strips()
                                    .contains(&long_strip)
                                    .not()
                        );

                        // long_strip.set_skip(false);
                        twin_cell
                            .faces
                            .get_face_mut(twin_face_index)
                            .get_strips_mut()
                            .push(long_strip.clone());

                        debug!(
                            "transit entity: {:?} long strip: {:?}",
                            twin_cell_address, long_strip
                        );

                        transit_segments.push(vertex_indices);

                        debug!("all_strip len: {}", all_strip.len());
                        if all_strip.is_empty() {
                            break;
                        } else {
                            debug!(
                            "three all_strip: len {} added_in_iteration {}, long_strip is loop: {}",
                            all_strip.len(),
                            added_in_iteration,
                            long_strip.get_loop()
                        );
                        }
                    }

                    if transit_segments.is_empty().not() {
                        twin_cell
                            .faces
                            .get_face_mut(twin_face_index)
                            .set_transit_segs(transit_segments.clone());
                        debug!(
                            "transit entity: {:?} transit segments: {:?}",
                            twin_cell_address, transit_segments
                        );
                    }

                    all_strip.clear();
                }
            }
        }
        debug!(
            "edit_transitional_face end: num: {}",
            self.transit_face_cells.len()
        );
    }

    fn traverse_face(
        cell_addresses: &HashMap<VoxelAddress, Cell>,
        cell_address: VoxelAddress,
        face_index: FaceIndex,
        strips: &mut Vec<Strip>,
    ) {
        if let Some(cell) = cell_addresses.get(&cell_address) {
            let face = cell.faces.get_face(face_index);
            match face.get_face_type() {
                FaceType::BranchFace => {
                    let sub_cell_indices = FACE_2_SUBCELL[face_index as usize];

                    for sub_cell_index in sub_cell_indices.iter() {
                        let child_address = cell.get_address().get_child_address(*sub_cell_index);
                        Self::traverse_face(cell_addresses, child_address, face_index, strips);
                    }
                }
                FaceType::LeafFace => {
                    for strip in face.get_strips().iter() {
                        debug!("traverse_face strip: {:?}", strip);
                        if strip.get_vertex_index(0).is_none() {
                            continue;
                        }

                        if strip.get_skip().not() {
                            debug!("traverse_face strip push: {:?}", strip);
                            strips.push(strip.clone());
                            debug!("transit entity: {:?}", cell_address);
                        }
                    }
                }
                FaceType::TransitFace => debug_assert!(false),
            }
        }
    }

    pub fn trace_component(&mut self) {
        info_span!("trace_component");
        info!("trace_component");
        for cell_address in self.leaf_cells.iter() {
            let mut cell_strips = Vec::new();
            let mut transit_segments = Vec::new();
            let mut components = Vec::new();

            let Some(cell) = self.cell_addresses.get(cell_address) else {
                continue;
            };

            // 获取一个cell的所有strip
            self.collect_strips(cell, &mut cell_strips, &mut transit_segments);

            // todo: transit segments number is not correct

            // 循环是为了建立多个Component
            loop {
                if cell_strips.is_empty() {
                    break;
                }

                Self::link_strips(&mut components, &mut cell_strips, &mut transit_segments);

                let Some(cell) = self.cell_addresses.get_mut(cell_address) else {
                    continue;
                };

                info!("trace component: components: {:?}", components);
                if components.is_empty().not() && cell.components.is_none() {
                    cell.components = Some(Vec::new());
                }
                cell.components.as_mut().map(|cell_components| {
                    cell_components.push(components.clone());
                    info!("trace component: push components: {:?}", components);
                    cell_components
                });

                info!(
                    "trace component: after add components: {:?}",
                    cell.components
                );

                components.clear();
            }
        }
    }

    fn collect_strips(
        &self,
        cell: &Cell,
        cell_strips: &mut Vec<Strip>,
        transit_segments: &mut Vec<Vec<u32>>,
    ) {
        for face_index in FaceIndex::iter() {
            let face = cell.faces.get_face(face_index);
            match face.get_face_type() {
                FaceType::BranchFace => {
                    debug_assert!(false);
                }
                FaceType::LeafFace => {
                    debug!("collect strip leaf face");
                    for strip in face.get_strips().iter() {
                        debug!(
                            "entity {:?}, strip: {:?}",
                            self.cell_addresses.get(cell.get_address()),
                            strip
                        );
                        if strip.get_vertex_index(0).is_some() {
                            cell_strips.push(strip.clone());
                        } else {
                            debug!("collect strip leaf face vertex index is none")
                        }
                    }
                    debug!("collect strip leaf face end");
                }
                FaceType::TransitFace => {
                    debug!("collect strip transit face");
                    debug_assert!(face.get_transit_segs().is_empty());
                    for strip in face.get_strips().iter() {
                        if strip.get_vertex_index(0).is_some() {
                            debug!(
                                "entity {:?}, strip: {:?}",
                                self.cell_addresses.get(cell.get_address()),
                                strip
                            );
                            cell_strips.push(strip.clone());
                        } else {
                            debug!(
                                "collect strip transit face vertex index is none: {:?}",
                                strip
                            )
                        }
                    }

                    let (twin_address, twin_face_index) = cell.get_twin_face_address(face_index);
                    let cell = self.cell_addresses.get(&twin_address).unwrap();
                    debug!("twin_address: {:?}, entity: {:?}", twin_address, cell);

                    let twin = cell.faces.get_face(twin_face_index);
                    debug!("twin.get_strips(): {:?}", twin.get_strips());
                    for strip in twin.get_strips().iter() {
                        if strip.get_vertex_index(0).is_some() {
                            cell_strips.push(strip.clone());
                        }
                    }

                    debug!("twin.get_transit_segments(): {:?}", twin.get_transit_segs());
                    for seg in twin.get_transit_segs().iter() {
                        transit_segments.push(seg.clone());
                    }

                    // debug_assert!(transit_segments.len() > 0);

                    debug!("collect strip transit face end");
                }
            }
        }

        debug_assert!(cell_strips.is_empty().not());
    }

    fn link_strips(
        components: &mut Vec<u32>,
        cell_strips: &mut Vec<Strip>,
        transit_segments: &mut Vec<Vec<u32>>,
    ) {
        debug_assert!(components.is_empty());
        debug_assert!(cell_strips[0].get_vertex_index(0).is_some());

        let mut added_in_iteration;
        let mut backwards = false;

        (0..cell_strips.len()).for_each(|i| {
            debug!("link_strips: {:?}", cell_strips[i].get_vertex());
        });
        (0..transit_segments.len()).for_each(|i| {
            debug!("link_transit_seg: {:?}", transit_segments[i]);
        });

        components.push(cell_strips[0].get_vertex_index(0).unwrap());

        loop {
            added_in_iteration = 0;

            cell_strips.retain(|strip| {
                debug_assert!(
                    strip.get_vertex_index(0).is_some() && strip.get_vertex_index(1).is_some()
                );

                let s_data0 = strip.get_vertex_index(0);
                let s_data1 = strip.get_vertex_index(1);

                match components.last() {
                    Some(v) if Some(*v) == s_data0 => {
                        backwards = false;
                        let mut transit = false;

                        if transit_segments.is_empty().not() {
                            Self::insert_data_from_twin(
                                components,
                                transit_segments,
                                strip,
                                &mut transit,
                                &mut added_in_iteration,
                                &backwards,
                            );
                            // if debug {
                            debug!("component transit: {:?}", components);
                            // }
                        }

                        if transit.not() {
                            if let Some(data) = s_data1 {
                                components.push(data);
                                added_in_iteration += 1;
                                // if debug {
                                debug!("component non transit: {:?}", components);
                                // }
                            }
                        }
                    }
                    Some(v) if Some(*v) == s_data1 => {
                        backwards = true;
                        let mut transit = false;

                        if transit_segments.is_empty().not() {
                            Self::insert_data_from_twin(
                                components,
                                transit_segments,
                                strip,
                                &mut transit,
                                &mut added_in_iteration,
                                &backwards,
                            );
                            // if debug {
                            debug!("component transit 2: {:?}", components);
                            // }
                        }

                        if transit.not() {
                            if let Some(data) = s_data0 {
                                components.push(data);
                                added_in_iteration += 1;
                                // if debug {
                                debug!("component non transit 2: {:?}", components);
                                // }
                            }
                        }
                    }
                    _ => {
                        // if debug {
                        debug!("component not add: {:?}", components);
                        // }
                        return true;
                    }
                }
                false
            });

            debug!(
                "component {:?} and first == last: {}",
                components,
                components.first() == components.last()
            );
            if components.first() == components.last() {
                components.remove(0);
            }
            debug!("component {:?} after", components,);

            if added_in_iteration <= 0 {
                break;
            }
        }

        loop {
            added_in_iteration = 0;

            cell_strips.retain(|strip| {
                debug_assert!(
                    strip.get_vertex_index(0).is_some() && strip.get_vertex_index(1).is_some()
                );

                let s_data0 = strip.get_vertex_index(0);
                let s_data1 = strip.get_vertex_index(1);

                match components.first() {
                    Some(v) if Some(*v) == s_data0 => {
                        backwards = false;
                        let mut transit = false;

                        if transit_segments.is_empty().not() {
                            Self::insert_data_from_twin(
                                components,
                                transit_segments,
                                strip,
                                &mut transit,
                                &mut added_in_iteration,
                                &backwards,
                            );
                            // if debug {
                            debug!("component transit 3: {:?}", components);
                            // }
                        }

                        if transit.not() {
                            if let Some(data) = s_data1 {
                                components.insert(0, data);
                                added_in_iteration += 1;
                                // if debug {
                                debug!("component non transit 3: {:?}", components);
                                // }
                            }
                        }
                    }
                    Some(v) if Some(*v) == s_data1 => {
                        backwards = true;
                        let mut transit = false;

                        if transit_segments.is_empty().not() {
                            Self::insert_data_from_twin(
                                components,
                                transit_segments,
                                strip,
                                &mut transit,
                                &mut added_in_iteration,
                                &backwards,
                            );
                            // if debug {
                            debug!("component transit 4: {:?}", components);
                            // }
                        }

                        if transit.not() {
                            if let Some(data) = s_data0 {
                                components.insert(0, data);
                                added_in_iteration += 1;
                                // if debug {
                                debug!("component non transit 4: {:?}", components);
                                // }
                            }
                        }
                    }
                    _ => {
                        // if debug {
                        debug!("component 2 not add: {:?}", components);
                        // }
                        return true;
                    }
                }
                false
            });

            debug!(
                "component {:?} and first == last: {}",
                components,
                components.first() == components.last()
            );
            if components.first() == components.last() {
                components.remove(0);
            }
            debug!("component {:?} after", components,);

            if added_in_iteration <= 0 {
                break;
            }
        }

        debug_assert!(components.first() != components.last());
        debug_assert!(components.len() >= 3);
    }

    fn insert_data_from_twin(
        components: &mut Vec<u32>,
        transit_segments: &[Vec<u32>],
        strip: &Strip,
        transit: &mut bool,
        added_in_iteration: &mut i32,
        backwards: &bool,
    ) {
        for seg in transit_segments.iter() {
            if Self::compare_strip_to_seg(strip, seg) {
                // if debug {
                //     debug!(
                //         "success component cell_strips: {:?} {:?}",
                //         strip.get_vertex_index(0),
                //         strip.get_vertex_index(1)
                //     );
                //     debug!("success component seg: {:?}", seg,);
                // }
                if *backwards {
                    for i in (0..seg.len()).rev() {
                        components.push(seg[i]);
                    }
                } else {
                    (0..seg.len()).for_each(|i| {
                        components.push(seg[i]);
                    });
                }
                // transit_segments.remove(i);
                *added_in_iteration += 1;
                *transit = true;
                break;
            } else {
                // if debug {
                //     debug!(
                //         "fail component cell_strips: {:?} {:?}",
                //         strip.get_vertex_index(0),
                //         strip.get_vertex_index(1)
                //     );
                //     debug!("fail component seg: {:?}", seg,);
                // }
            }
        }
    }

    fn compare_strip_to_seg(strip: &Strip, seg: &[u32]) -> bool {
        let s0 = strip.get_vertex_index(0);
        let s1 = strip.get_vertex_index(1);

        (seg.first() == s0.as_ref() && seg.last() == s1.as_ref())
            || (seg.first() == s1.as_ref() && seg.last() == s0.as_ref())
    }
}
