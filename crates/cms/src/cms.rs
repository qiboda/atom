use std::{cell::RefCell, ops::Range, rc::Rc};

use bevy::prelude::info;
use nalgebra::Vector3;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    iso_surface::IsoSurface,
    mesh::{Mesh, Vertex},
    octree::{
        cell::{Cell, CellType},
        edge_block::VertexIndices,
        face::{Face, FaceType},
        octree::Octree,
        point::Point,
        strip::Strip,
        tables::{
            Direction, Face2DEdge, Face2DVertex, FaceIndex, SubCellIndex, SubFaceIndex, EDGE_MAP,
            FACE_VERTEX, VERTEX_MAP,
        },
    },
    sample::sample_range_3d::SampleRange3D,
    MAX_OCTREE_RES,
};

/// Cubical Marching Squares
pub struct CMS {
    iso_level: f32,
    negative_inside: bool,

    container: Vector3<(f32, f32)>,
    /// neighbours sample position offset
    offset: Vector3<f32>,

    sample_fn: Rc<dyn IsoSurface>,
    sample_size: Vector3<usize>,
    sample_data: SampleRange3D<f32>,

    vertex_index_data: SampleRange3D<VertexIndices>,

    vertices: Vec<Vertex>,

    octree: Box<Octree>,
    octree_root: Option<Rc<RefCell<Cell>>>,

    snap_centro_id: bool,
}

impl CMS {
    pub fn new(
        iso_level: f32,
        container: Vector3<(f32, f32)>,
        sample_fn: Rc<dyn IsoSurface>,
    ) -> Self {
        info!("CMS::new");
        let mut sample_size = Vector3::new(0, 0, 0);
        sample_size.x = 2usize.pow(MAX_OCTREE_RES as u32) + 1;
        sample_size.y = sample_size.x;
        sample_size.z = sample_size.x;

        let offset = Vector3::new(
            (container.x.1 - container.x.0) / (sample_size.x - 1) as f32,
            (container.y.1 - container.y.0) / (sample_size.y - 1) as f32,
            (container.z.1 - container.z.0) / (sample_size.z - 1) as f32,
        );

        let mut sample_data = SampleRange3D::default();
        sample_data.resize(container, sample_size);

        info!("CMS::sample size start");
        for i in 0..sample_size.x {
            let x = container.x.0 + i as f32 * offset.x + iso_level;
            for j in 0..sample_size.y {
                let y = container.y.0 + j as f32 * offset.y + iso_level;
                for k in 0..sample_size.z {
                    let z = container.z.0 + k as f32 * offset.z + iso_level;

                    let value = sample_fn.get_value(x, y, z);
                    sample_data.set_value(i, j, k, value);
                }
            }
        }
        info!("CMS::sample size end");

        let mut edge_data = SampleRange3D::default();
        edge_data.resize(container, sample_size);

        let octree = Box::new(Octree::new(
            sample_size,
            sample_data.clone(),
            offset,
            sample_fn.clone(),
        ));

        CMS {
            iso_level,
            negative_inside: true,
            container,
            offset,
            sample_fn,
            sample_size,
            sample_data,
            vertex_index_data: edge_data,
            octree,
            vertices: vec![],
            octree_root: None,
            snap_centro_id: false,
        }
    }

    pub fn initialize(&mut self) {
        // todo: add iso_level or not?

        self.octree.build_octree();
        self.octree_root = self.octree.get_root();
    }
}

impl CMS {
    pub fn sample_size(&self) -> &Vector3<usize> {
        &self.sample_size
    }
}

impl IsoSurface for CMS {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        self.sample_fn.get_value(x, y, z)
    }

    fn set_iso_level(&mut self, iso_level: f32) {
        self.iso_level = iso_level;
    }

    fn get_iso_level(&self) -> f32 {
        self.iso_level
    }

    fn set_negative_inside(&mut self, negative_inside: bool) {
        self.negative_inside = negative_inside;
    }

    fn is_negative_inside(&self) -> bool {
        self.negative_inside
    }
}

impl CMS {
    pub fn extract_surface(&mut self, mesh: &mut Mesh) {
        self.cubical_marching_sqaures_algorithm();
        match &self.octree_root {
            Some(root) => {
                info!("tessellation_traversal");
                self.tessellation_traversal(root.clone(), mesh);
                info!("create_mesh");
                self.create_mesh(mesh);
            }
            None => {}
        }
    }
}

impl CMS {
    pub fn cubical_marching_sqaures_algorithm(&mut self) {
        self.generate_segments(self.octree_root.clone());

        self.edit_transitional_face();

        self.trace_comonent();
    }

    /// 生成每个面的连线，以及边的顶点的位置信息。
    pub fn generate_segments(&mut self, cell: Option<Rc<RefCell<Cell>>>) {
        if let Some(cell) = cell {
            match cell.borrow().get_cell_type() {
                CellType::Branch => {
                    for subcell_index in SubCellIndex::iter() {
                        self.generate_segments(cell.borrow().get_child(subcell_index));
                    }
                }
                CellType::Leaf => {
                    let mut indices = [Vector3::zeros(); Face2DVertex::COUNT];
                    for face_index in FaceIndex::iter() {
                        for (i, face_vertex) in Face2DVertex::iter().enumerate() {
                            let vertex_pos = FACE_VERTEX[face_index as usize][face_vertex as usize];
                            indices[i] =
                                cell.borrow().get_corner_sample_index()[vertex_pos as usize];
                        }
                        cell.borrow()
                            .get_face(face_index)
                            .borrow_mut()
                            .get_strips_mut()
                            .resize(2, Strip::default());
                        self.make_face_segments(&indices, cell.borrow().get_face(face_index));
                    }
                }
            }
        }
    }

    // make segments in a face.
    pub fn make_face_segments(&mut self, indices: &[Vector3<usize>; 4], face: Rc<RefCell<Face>>) {
        let edges = (0..4).fold(0, |acc, i| {
            acc | if self
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
            self.make_strip(e0a, e0b, indices, face.clone(), 0)
        }

        let e1a = EDGE_MAP[edges as usize][1][0];
        let e1b = EDGE_MAP[edges as usize][1][1];

        if e1a.is_some() {
            self.make_strip(e1a, e1b, indices, face.clone(), 1);
        }
    }

    pub fn make_strip(
        &mut self,
        edge0: Option<Face2DEdge>,
        edge1: Option<Face2DEdge>,
        indices: &[Vector3<usize>; 4],
        face: Rc<RefCell<Face>>,
        strip_index: usize,
    ) {
        assert!(edge0.is_some() && edge1.is_some());

        let mut s = Strip::new(edge0, edge1);

        self.populate_strip(&mut s, indices, 0);
        self.populate_strip(&mut s, indices, 1);

        face.borrow_mut().get_strips_mut()[strip_index] = s.clone();
    }

    /// 计算strip的一条边的顶点信息
    pub fn populate_strip(
        &mut self,
        strip: &mut Strip,
        indices: &[Vector3<usize>; 4],
        edge_index: usize,
    ) {
        let edge = strip.get_edge(edge_index);
        assert!(edge.is_some());

        let vertex0 = VERTEX_MAP[edge.unwrap() as usize][0];
        let vertex1 = VERTEX_MAP[edge.unwrap() as usize][1];

        let vertex_coord0 = indices[vertex0 as usize];
        let vertex_coord1 = indices[vertex1 as usize];

        let mut vertex_range = Range::default();

        let edge_dir = self.get_edges_betwixt(&mut vertex_range, vertex_coord0, vertex_coord1);
        let edge_dir_index = edge_dir as usize;

        let sign_change_dir_coord = self.exact_sign_change_index(
            vertex_range.clone(),
            edge_dir,
            vertex_coord0,
            vertex_coord1,
        );
        assert!(vertex_range.contains(&sign_change_dir_coord));

        let mut crossing_index_0 = vertex_coord0;
        let mut crossing_index_1 = vertex_coord1;

        crossing_index_0[edge_dir_index] = sign_change_dir_coord;
        crossing_index_1[edge_dir_index] = sign_change_dir_coord + 1;

        assert!(
            self.sample_data
                .get_value(crossing_index_0.x, crossing_index_0.y, crossing_index_0.z)
                * self.sample_data.get_value(
                    crossing_index_1.x,
                    crossing_index_1.y,
                    crossing_index_1.z
                )
                <= 0.0
        );

        let mut dupli = false;

        let value_0 = self.vertex_index_data.get_value(
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
            self.make_vertex(
                strip,
                edge_dir,
                crossing_index_0,
                crossing_index_1,
                edge_index,
            );
        }
    }

    /// 获取边的两个顶点的方向以及距离。
    pub fn get_edges_betwixt(
        &mut self,
        range: &mut Range<usize>,
        vertex_coord0: Vector3<usize>,
        vertex_coord1: Vector3<usize>,
    ) -> Direction {
        let mut direction = None;

        let diff = (vertex_coord0.cast::<i32>() - vertex_coord1.cast::<i32>()).abs();

        if diff.x > 0 {
            range.start = vertex_coord0.x.min(vertex_coord1.x) as usize;
            range.end = vertex_coord0.x.max(vertex_coord1.x) as usize;
            direction = Some(Direction::XAxis);
        } else if diff.y > 0 {
            range.start = vertex_coord0.y.min(vertex_coord1.y) as usize;
            range.end = vertex_coord0.y.max(vertex_coord1.y) as usize;
            direction = Some(Direction::YAxis);
        } else if diff.z > 0 {
            range.start = vertex_coord0.z.min(vertex_coord1.z) as usize;
            range.end = vertex_coord0.z.max(vertex_coord1.z) as usize;
            direction = Some(Direction::ZAxis);
        }

        assert!(direction.is_some());

        return direction.unwrap();
    }

    /// 检测是否有精确的符号变化。
    /// 返回值为符号变化的前一个索引。
    pub fn exact_sign_change_index(
        &mut self,
        vertex_range: Range<usize>,
        edge_dir: Direction,
        vertex_coord0: Vector3<usize>,
        vertex_coord1: Vector3<usize>,
    ) -> usize {
        let mut start_vertex_coord = Vector3::new(usize::MAX, usize::MAX, usize::MAX);

        if vertex_coord0[edge_dir as usize] == vertex_range.start {
            start_vertex_coord = vertex_coord0;
        } else if vertex_coord1[edge_dir as usize] == vertex_range.start {
            start_vertex_coord = vertex_coord1;
        }

        // 因为传入的两个顶点是Strip的顶点，所以不可能符号相等。
        if vertex_range.end - vertex_range.start == 1 {
            let this_value = self.sample_data.get_value(
                start_vertex_coord.x,
                start_vertex_coord.y,
                start_vertex_coord.z,
            );
            let mut end_vertex_coord = start_vertex_coord;
            end_vertex_coord[edge_dir as usize] = start_vertex_coord[edge_dir as usize] + 1;
            let next_value = self.sample_data.get_value(
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

            let this_value = self.sample_data.get_value(indexer.x, indexer.y, indexer.z);

            indexer[edge_dir as usize] = i + 1;
            let next_value = self.sample_data.get_value(indexer.x, indexer.y, indexer.z);

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
        &mut self,
        strip: &mut Strip,
        edge_dir: Direction,
        crossing_index_0: Vector3<usize>,
        crossing_index_1: Vector3<usize>,
        edge_index: usize,
    ) {
        let pos0 =
            self.sample_data
                .get_pos(crossing_index_0.x, crossing_index_0.y, crossing_index_0.z);
        let value0 =
            self.sample_data
                .get_value(crossing_index_0.x, crossing_index_0.y, crossing_index_0.z);
        let point0 = Point::new_with_position_and_value(&pos0, value0);

        let pos1 =
            self.sample_data
                .get_pos(crossing_index_1.x, crossing_index_1.y, crossing_index_1.z);
        let value1 =
            self.sample_data
                .get_value(crossing_index_1.x, crossing_index_1.y, crossing_index_1.z);
        let point1 = Point::new_with_position_and_value(&pos1, value1);

        let crossing_point = self.find_crossing_point(2, &point0, &point1);
        let mut gradient = Vector3::new(0.0, 0.0, 0.0);
        self.find_gradient(&mut gradient, self.offset, &crossing_point);

        let vert = Vertex::new_with_position_and_normals(&crossing_point, &gradient);
        self.vertices.push(vert);

        strip.set_vertex_index(edge_index, self.vertices.len() - 1);
        strip.set_crossing_left_coord(edge_index, crossing_index_0);
        strip.set_edge_dir(edge_index, Some(edge_dir));

        let mut e = self.vertex_index_data.get_value(
            crossing_index_0.x,
            crossing_index_0.y,
            crossing_index_0.z,
        );

        assert!(e
            .get_vertex_index()
            .get(edge_dir as usize)
            .unwrap()
            .is_none());
        e.set_dir_vertex_index(edge_dir, self.vertices.len() - 1);
        self.vertex_index_data.set_value(
            crossing_index_0.x,
            crossing_index_0.y,
            crossing_index_0.z,
            e.clone(),
        );
    }

    /// @param quality iter count
    pub fn find_crossing_point(
        &mut self,
        quality: usize,
        point0: &Point,
        point1: &Point,
    ) -> Vector3<f32> {
        let iso_value = self.sample_fn.get_iso_level();

        let p0 = point0.get_position();
        let v0 = point0.get_value();

        let p1 = point1.get_position();
        let v1 = point1.get_value();

        let alpha = (iso_value - v0) / (v1 - v0);
        let mut pos = p0 + (p1 - p0) * alpha;
        let val = self.sample_fn.get_value(pos.x, pos.y, pos.z);

        let point = Point::new_with_position_and_value(&pos, val);

        // 误差足够小，或者迭代次数足够多，就认为找到了交点。
        if (iso_value - val).abs() < f32::EPSILON || quality == 0 {
            return pos;
        } else {
            if val < 0.0 {
                if v0 > 0.0 {
                    pos = self.find_crossing_point(quality - 1, &point, point0);
                } else if v1 > 0.0 {
                    pos = self.find_crossing_point(quality - 1, &point, point1);
                }
            } else {
                if v0 < 0.0 {
                    pos = self.find_crossing_point(quality - 1, point0, &point);
                } else if v1 < 0.0 {
                    pos = self.find_crossing_point(quality - 1, point1, &point);
                }
            }
        }

        return pos;
    }

    pub fn find_gradient(
        &mut self,
        normal: &mut Vector3<f32>,
        offset: Vector3<f32>,
        position: &Vector3<f32>,
    ) {
        let val = self.sample_fn.get_value(position.x, position.y, position.z);
        let dx = self
            .sample_fn
            .get_value(position.x + offset.x, position.y, position.z);

        let dy = self
            .sample_fn
            .get_value(position.x, position.y + offset.y, position.z);

        let dz = self
            .sample_fn
            .get_value(position.x, position.y, position.z + offset.z);

        *normal = Vector3::new(dx - val, dy - val, dz - val);
    }
}

impl CMS {
    /// 计算面的Twin的Strip的起点和重点，以及所经过的顶点。
    /// todo: 如果twin是由多个leaf Cell的面组成的，会重复吧，需要添加检测
    pub fn edit_transitional_face(&mut self) {
        info!("edit_transitional_face");
        let cells = self.octree.get_cells();

        for cell in cells.iter() {
            for face_index in FaceIndex::iter() {
                let face = cell.borrow().get_face(face_index);
                if face.borrow().get_face_type() == &FaceType::TransitFace {
                    assert!(face.borrow().get_twin().is_some());
                    assert!(
                        self.octree
                            .get_cell(
                                face.borrow()
                                    .get_twin()
                                    .as_ref()
                                    .unwrap()
                                    .borrow()
                                    .get_cell_id()
                            )
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .get_cell_type()
                            == &CellType::Branch
                    );

                    assert!(
                        face.borrow()
                            .get_twin()
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .get_face_type()
                            != &FaceType::TransitFace
                    );

                    let mut all_strips = Vec::new();
                    CMS::traverse_face(
                        &self,
                        face.borrow().get_twin().as_ref().unwrap().clone(),
                        &mut all_strips,
                    );

                    // info!("all_strips len: {}", all_strips.len(),);
                    // for strip in all_strips.iter() {
                    //     info!("strip value: {:?}", strip);
                    // }

                    // todo: fix this
                    // assert!(all_strips.len() != 0);
                    if all_strips.len() == 0 {
                        // face.borrow_mut().set_face_type(FaceType::LeafFace);
                        // info!("all_strips: len is 0");
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

                        face.borrow()
                            .get_twin()
                            .as_ref()
                            .unwrap()
                            .borrow_mut()
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

                    for i in transit_segs.iter() {
                        for j in i.iter() {
                            assert!((*j as usize) < self.vertices.len());
                        }
                    }

                    if transit_segs.len() != 0 {
                        face.borrow()
                            .get_twin()
                            .as_ref()
                            .unwrap()
                            .borrow_mut()
                            .set_transit_segs(transit_segs);
                    }

                    all_strips.clear();
                }
            }
        }
    }

    pub fn traverse_face(self: &Self, face: Rc<RefCell<Face>>, strips: &mut Vec<Strip>) {
        match face.borrow().get_face_type() {
            FaceType::BranchFace => {
                for (i, sub_face_index) in SubFaceIndex::iter().enumerate() {
                    if let Some(children) = face.borrow().get_children() {
                        if children[i].is_none() {
                            continue;
                        }
                        CMS::traverse_face(&self, children[i].as_ref().unwrap().clone(), strips);
                    }
                }
            }
            FaceType::LeafFace => {
                for strip in face.borrow().get_strips().iter() {
                    // info!(
                    //     "face id: {} leaf strip {:?}",
                    //     face.borrow().get_cell_id(),
                    //     strip
                    // );
                    // assert!(strip.get_vertex_index(0).is_none());
                    if strip.get_vertex_index(0).is_none() {
                        continue;
                    }
                    strips.push(strip.clone());
                }
            }
            FaceType::TransitFace => assert!(false),
        }
    }

    pub fn trace_comonent(&mut self) {
        info!("trace_comonent");
        let cells = self.octree.get_leaf_cells();
        // for i in 0..MAX_OCTREE_RES {
        for cell in cells.iter() {
            // if *cell.borrow().get_cur_subdiv_level() as usize == MAX_OCTREE_RES - i {
            if cell.borrow().get_cell_type() == &CellType::Leaf {
                let mut cell_strips = Vec::new();
                let mut transit_segs = Vec::new();
                let mut components = Vec::new();

                // 获取一个cell的所有strip
                CMS::collect_strips(&self, cell.clone(), &mut cell_strips, &mut transit_segs);

                // todo: transit segs number is not correct

                // info!("loop start {}", cell.borrow().get_id());
                // 循环是为了建立多个Component
                loop {
                    if cell_strips.len() == 0 {
                        break;
                    }

                    CMS::link_strips(&self, &mut components, &mut cell_strips, &mut transit_segs);

                    cell.borrow_mut()
                        .get_componnets_mut()
                        .push(components.clone());

                    components.clear();
                }
                // info!("loop end");
            }
            // }
            // }
        }
    }

    pub fn collect_strips(
        self: &Self,
        cell: Rc<RefCell<Cell>>,
        cell_strips: &mut Vec<Strip>,
        transit_segs: &mut Vec<Vec<usize>>,
    ) {
        for face in FaceIndex::iter() {
            let cell_borrow = cell.borrow();
            let face = cell_borrow.get_face(face);
            let face_borrow = face.borrow();
            match face_borrow.get_face_type() {
                FaceType::BranchFace => {
                    assert!(false);
                }
                FaceType::LeafFace => {
                    for strip in face_borrow.get_strips().iter() {
                        if strip.get_vertex_index(0).is_some() {
                            cell_strips.push(strip.clone());
                            // info!(
                            //     "leaf face: {:?} {:?}",
                            //     strip.get_vertex_index(0),
                            //     strip.get_vertex_index(1)
                            // );
                        }
                    }
                }
                FaceType::TransitFace => {
                    for strip in face_borrow.get_strips().iter() {
                        if strip.get_vertex_index(0).is_some() {
                            cell_strips.push(strip.clone());
                            // info!(
                            //     "TransitFace leaf face: {:?} {:?}",
                            //     strip.get_vertex_index(0),
                            //     strip.get_vertex_index(1)
                            // );
                        }
                    }

                    let twin = face_borrow.get_twin();
                    if twin.is_none() {
                        continue;
                    }

                    assert!(
                        twin.as_ref().unwrap().borrow().get_strips().len()
                            == twin.as_ref().unwrap().borrow().get_transit_segs().len()
                    );

                    for (i, strip) in twin
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .get_strips()
                        .iter()
                        .enumerate()
                    {
                        strip.get_vertex_index(0).map(|data| {
                            transit_segs.push(
                                twin.as_ref().unwrap().borrow().get_transit_segs()[i].clone(),
                            );
                            // info!("transit segs: {:?}", transit_segs);
                            cell_strips.push(strip.clone());
                            // info!(
                            //     "transit cell strips: {:?} {:?}",
                            //     strip.get_vertex_index(0),
                            //     strip.get_vertex_index(1)
                            // );
                        });
                    }
                }
            }
        }

        assert!(cell_strips.len() > 0);
    }

    pub fn link_strips(
        self: &Self,
        components: &mut Vec<usize>,
        cell_strips: &mut Vec<Strip>,
        transit_segs: &mut Vec<Vec<usize>>,
    ) {
        assert!(components.len() == 0);
        assert!(cell_strips[0].get_vertex_index(0).is_some());

        let mut added_in_iteration;
        let mut backwards = false;

        components.push(cell_strips[0].get_vertex_index(0).unwrap());

        // info!("len: {}", cell_strips.len());
        // for strip in cell_strips.iter() {
        //     info!(
        //         "component cell_strips: {:?} {:?}",
        //         strip.get_vertex_index(0),
        //         strip.get_vertex_index(1)
        //     );
        // }
        // info!("transit segs: {:?}", transit_segs);

        // if debug {
        //     info!("component start: {:?}", components);
        //     for strip in cell_strips.iter() {
        //         info!(
        //             "component cell_strips: {:?} {:?}",
        //             strip.get_vertex_index(0),
        //             strip.get_vertex_index(1)
        //         );
        //     }
        //     info!("transit segs: {:?}", transit_segs);
        // }

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
                            CMS::insert_data_from_twin(
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
                            CMS::insert_data_from_twin(
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
            for comp in components.iter() {
                assert!(comp < &self.vertices.len());
            }

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
                            CMS::insert_data_from_twin(
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
                            CMS::insert_data_from_twin(
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

            for comp in components.iter() {
                assert!(comp < &self.vertices.len());
            }

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
            if CMS::compare_strip_to_seg(strip, seg) {
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
}

impl CMS {
    pub fn tessellation_traversal(&mut self, cell: Rc<RefCell<Cell>>, mesh: &mut Mesh) {
        let cell_type = cell.borrow().get_cell_type().clone();
        match cell_type {
            CellType::Branch => {
                for subcell_index in SubCellIndex::iter() {
                    // assert!(cell.borrow().get_child(subcell_index).is_some());
                    if let Some(cell) = cell.borrow().get_child(subcell_index) {
                        self.tessellation_traversal(cell, mesh);
                    }
                }
            }
            CellType::Leaf => {
                for component in cell.borrow_mut().get_componnets_mut().iter_mut() {
                    self.tessellate_component(mesh, component);
                }
            }
        }
    }

    pub fn tessellate_component(&mut self, mesh: &mut Mesh, component: &mut Vec<usize>) {
        let mut centro_id = Vertex::new();

        let num_of_indices = component.len();

        assert!(num_of_indices >= 3);

        if num_of_indices == 3 {
            CMS::make_tri(mesh, component);
        } else if num_of_indices > 3 {
            let mut centro_id_pos = Vector3::new(0.0, 0.0, 0.0);
            let mut centro_id_normal = Vector3::new(0.0, 0.0, 0.0);

            for comp in component.iter() {
                let vertex = &self.vertices[*comp];
                centro_id_pos += vertex.get_position();
                centro_id_normal += vertex.get_normals();
            }

            let mut med_vertex = Vector3::new(
                centro_id_pos.x / num_of_indices as f32,
                centro_id_pos.y / num_of_indices as f32,
                centro_id_pos.z / num_of_indices as f32,
            );

            if self.snap_centro_id {
                let med_val = self
                    .sample_fn
                    .get_value(med_vertex.x, med_vertex.y, med_vertex.z);

                let med_dimension = self.offset / 2.0;

                let mut med_gradient = Vector3::new(0.0, 0.0, 0.0);

                CMS::find_gradient_with_value(
                    self,
                    &mut med_gradient,
                    &med_dimension,
                    &med_vertex,
                    med_val,
                );

                // 沿着反方向偏移
                med_vertex += -med_gradient * med_val;
            }

            centro_id.set_position(&med_vertex);

            centro_id_normal.normalize_mut();

            centro_id.set_normals(&centro_id_normal);

            self.vertices.push(centro_id);

            component.push(self.vertices.len() - 1);

            CMS::make_tri_fan(mesh, component);
        }
    }

    /// 3d梯度等价于2d的法线, 3d梯度等于2d方向上最快的变化方向，也就是法线
    /// 4d梯度等价于3d的法线, 4d梯度等于3d方向上最快的变化方向，也就是法线
    pub fn find_gradient_with_value(
        self: &Self,
        normal: &mut Vector3<f32>,
        dimensions: &Vector3<f32>,
        position: &Vector3<f32>,
        value: f32,
    ) {
        let dx = self
            .sample_fn
            .get_value(position.x + dimensions.x, position.y, position.z);

        let dy = self
            .sample_fn
            .get_value(position.x, position.y + dimensions.y, position.z);

        let dz = self
            .sample_fn
            .get_value(position.x, position.y, position.z + dimensions.z);

        *normal = Vector3::new(dx - value, dy - value, dz - value);

        normal.normalize_mut();
    }

    pub fn make_tri(mesh: &mut Mesh, component: &Vec<usize>) {
        for i in 0..3 {
            mesh.get_indices_mut().push(component[i] as u32);
        }
    }

    // 扇形三角面
    pub fn make_tri_fan(mesh: &mut Mesh, component: &Vec<usize>) {
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
}

impl CMS {
    pub fn create_mesh(&self, mehs: &mut Mesh) {
        for vertex in self.vertices.iter() {
            mehs.get_vertices_mut().push(vertex.clone());
        }
    }
}
