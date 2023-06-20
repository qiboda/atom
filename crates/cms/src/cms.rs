use std::{cell::RefCell, ops::Range, rc::Rc};

use nalgebra::Vector3;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    iso_surface::IsoSurface,
    mesh::{Mesh, Vertex},
    octree::{
        cell::{Cell, CellType},
        edge_block::EdgeBlock,
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

pub struct CMS {
    iso_level: f32,
    negative_inside: bool,

    container: Vector3<(f32, f32)>,
    /// neighbours sample position offset
    offset: Vector3<f32>,

    sample_fn: Rc<dyn IsoSurface>,
    sample_size: Vector3<usize>,
    sample_data: SampleRange3D<f32>,

    edge_data: SampleRange3D<EdgeBlock>,

    vertices: Vec<Vertex>,

    octree: Box<Octree>,
    octree_root: Option<Rc<RefCell<Cell>>>,

    desired_cells: Vec<usize>,

    snap_centro_id: bool,
}

impl CMS {
    pub fn new(
        iso_level: f32,
        container: Vector3<(f32, f32)>,
        sample_fn: Rc<dyn IsoSurface>,
    ) -> Self {
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
            edge_data,
            octree,
            vertices: vec![],
            octree_root: None,
            desired_cells: Vec::new(),
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
                self.tessellation_traversal(root.clone(), mesh);
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

        let mut s = Strip::new(false, edge0, edge1);

        self.populate_strip(&mut s, indices, 0);
        self.populate_strip(&mut s, indices, 1);

        face.borrow_mut().get_strips_mut()[strip_index] = s;
        face.borrow_mut().set_skip(false);
    }

    pub fn populate_strip(
        &mut self,
        strip: &mut Strip,
        indices: &[Vector3<usize>; 4],
        index: usize,
    ) {
        let edge = strip.get_edge(index);
        assert!(edge.is_some());

        let index0 = VERTEX_MAP[edge.unwrap() as usize][0];
        let index1 = VERTEX_MAP[edge.unwrap() as usize][1];

        let index0 = indices[index0 as usize];
        let index1 = indices[index1 as usize];

        let mut range = Range { start: 0, end: 0 };

        let dir = self.get_edges_betwixt(&mut range, index0, index1);
        let dir_index = dir as usize;
        assert!((index0[dir_index] as i32 - index1[dir_index] as i32).abs() > 0);
        assert!((index0[dir_index] == range.start) || (index0[dir_index] == range.end));
        assert!((index1[dir_index] == range.start) || (index1[dir_index] == range.end));

        let sign_change = self.exact_sign_change_index(range.clone(), dir, index0, index1);
        assert!(range.contains(&sign_change));

        let mut crossing_index_0 = index0;
        let mut crossing_index_1 = index1;

        crossing_index_0[dir_index] = sign_change;
        crossing_index_1[dir_index] = sign_change + 1;

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

        let value_0 =
            self.edge_data
                .get_value(crossing_index_0.x, crossing_index_0.y, crossing_index_0.z);
        if value_0.is_empty() == false {
            if value_0.get_edge_index().get(dir_index).is_some() {
                strip.set_data(
                    index,
                    value_0.get_edge_index().get(dir_index).unwrap().unwrap() as i8,
                );
                strip.set_block(index, crossing_index_0);
                strip.set_dir(index, Some(dir));
                dupli = true;
            }
        }

        if dupli == false {
            self.make_vertex(strip, dir, crossing_index_0, crossing_index_1, index);
        }
    }

    /// 获取边缘中间
    pub fn get_edges_betwixt(
        &mut self,
        range: &mut Range<usize>,
        index0: Vector3<usize>,
        index1: Vector3<usize>,
    ) -> Direction {
        let mut direction = None;

        let diff = (index0.cast::<i32>() - index1.cast::<i32>()).abs();

        if diff.x > 0 {
            range.start = index0.x.min(index1.x) as usize;
            range.end = index0.x.max(index1.x) as usize;
            direction = Some(Direction::XAxis);
        } else if diff.y > 0 {
            range.start = index0.y.min(index1.y) as usize;
            range.end = index0.y.max(index1.y) as usize;
            direction = Some(Direction::YAxis);
        } else if diff.z > 0 {
            range.start = index0.z.min(index1.z) as usize;
            range.end = index0.z.max(index1.z) as usize;
            direction = Some(Direction::ZAxis);
        }

        assert!(direction.is_some());

        return direction.unwrap();
    }

    pub fn exact_sign_change_index(
        &mut self,
        range: Range<usize>,
        dir: Direction,
        index0: Vector3<usize>,
        index1: Vector3<usize>,
    ) -> usize {
        let mut first_index = Vector3::new(usize::MAX, usize::MAX, usize::MAX);

        if index0[dir as usize] == range.start {
            first_index = index0;
        } else if index1[dir as usize] == range.start {
            first_index = index1;
        }

        if range.end - range.start == 1 {
            return first_index[dir as usize];
        }

        let mut indexer = first_index;

        for i in range {
            indexer[dir as usize] = i;

            let this_value = self.sample_data.get_value(indexer.x, indexer.y, indexer.z);

            indexer[dir as usize] = i + 1;
            let next_value = self.sample_data.get_value(indexer.x, indexer.y, indexer.z);

            if this_value * next_value < 0.0 {
                return i;
            }
        }

        assert!(false);

        return usize::MAX;
    }

    pub fn make_vertex(
        &mut self,
        strip: &mut Strip,
        dir: Direction,
        crossing_index_0: Vector3<usize>,
        crossing_index_1: Vector3<usize>,
        index: usize,
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
        let mut normal = Vector3::new(0.0, 0.0, 0.0);
        self.find_gradient(&mut normal, self.offset, &crossing_point);

        let vert = Vertex::new_with_position_and_normals(&crossing_point, &normal);
        self.vertices.push(vert);

        strip.set_data(index, (self.vertices.len() - 1) as i8);
        strip.set_block(index, crossing_index_0);
        strip.set_dir(index, Some(dir));

        let mut e =
            self.edge_data
                .get_value(crossing_index_0.x, crossing_index_0.y, crossing_index_0.z);

        if e.is_empty() {
            e.set_empty();
        }
        assert!(e.get_edge_index().get(index).is_none());
        e.set_dir_edge_index(dir, self.vertices.len() - 1);
        self.edge_data.set_value(
            crossing_index_0.x,
            crossing_index_0.y,
            crossing_index_0.z,
            e.clone(),
        );
    }

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

        if (iso_value - val).abs() < 0.0 || quality == 0 {
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
    pub fn edit_transitional_face(&mut self) {
        let cells = self.octree.get_cells();

        for cell in cells.iter() {
            for face_index in FaceIndex::iter() {
                let face = cell.borrow().get_face(face_index);
                if face.borrow().get_face_type() == &FaceType::TransitFace {
                    let face_borrow = face.borrow();
                    assert!(face_borrow.get_twin().is_some());
                    assert!(
                        self.octree
                            .get_cell(
                                face_borrow
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
                        face_borrow
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
                        face_borrow.get_twin().as_ref().unwrap().clone(),
                        &mut all_strips,
                    );

                    if all_strips.len() == 0 {
                        face.borrow_mut().set_face_type(FaceType::LeafFace);
                        continue;
                    }

                    let mut transit_segs = Vec::new();

                    loop {
                        let mut vertex_indices = Vec::new();

                        vertex_indices.push(all_strips[0].get_data(0));
                        vertex_indices.push(all_strips[0].get_data(1));

                        let mut long_strip = all_strips[0].clone();

                        all_strips.remove(0);

                        let mut added_in_iteration;

                        loop {
                            added_in_iteration = 0;

                            for strip in all_strips.iter() {
                                if vertex_indices[vertex_indices.len() - 1] == strip.get_data(0) {
                                    vertex_indices.push(strip.get_data(1));
                                    long_strip.change_back(strip, 1);
                                    added_in_iteration += 1;
                                } else if vertex_indices[vertex_indices.len() - 1]
                                    == strip.get_data(1)
                                {
                                    vertex_indices.push(strip.get_data(0));
                                    long_strip.change_back(strip, 0);
                                    added_in_iteration += 1;
                                } else {
                                    continue;
                                }

                                // all_strips.remove(i);

                                if vertex_indices[0] == vertex_indices[vertex_indices.len() - 1] {
                                    vertex_indices.remove(0);
                                    long_strip.set_loop(true);
                                }
                            }

                            if all_strips.len() <= 0
                                || added_in_iteration <= 0
                                || long_strip.get_loop()
                            {
                                break;
                            }
                        }

                        if long_strip.get_loop() == false && all_strips.len() > 0 {
                            loop {
                                added_in_iteration = 0;

                                for strip in all_strips.iter() {
                                    if vertex_indices[0] == strip.get_data(0) {
                                        vertex_indices.insert(0, strip.get_data(1));
                                        long_strip.change_front(strip, 1);
                                        added_in_iteration += 1;
                                    } else if vertex_indices[0] == strip.get_data(1) {
                                        vertex_indices.insert(0, strip.get_data(0));
                                        long_strip.change_front(strip, 0);
                                        added_in_iteration += 1;
                                    } else {
                                        continue;
                                    }
                                }

                                // all_strips.remove(i);

                                if vertex_indices[0] == vertex_indices[vertex_indices.len() - 1] {
                                    vertex_indices.remove(0);
                                    long_strip.set_loop(true);
                                }

                                if all_strips.len() <= 0
                                    || added_in_iteration <= 0
                                    || long_strip.get_loop()
                                {
                                    break;
                                }
                            }
                        }

                        long_strip.set_skip(false);

                        face_borrow
                            .get_twin()
                            .as_ref()
                            .unwrap()
                            .borrow_mut()
                            .get_strips_mut()
                            .push(long_strip.clone());

                        transit_segs.push(vertex_indices);

                        if all_strips.len() == 0 {
                            break;
                        }
                    }

                    for i in transit_segs.iter() {
                        for j in i.iter() {
                            assert!((*j as usize) < self.vertices.len());
                        }
                    }

                    if transit_segs.len() != 0 {
                        face_borrow
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

    pub fn traverse_face(self: &Self, face: Rc<RefCell<Face>>, transit_strips: &mut Vec<Strip>) {
        match face.borrow().get_face_type() {
            FaceType::BranchFace => {
                for i in SubFaceIndex::iter() {
                    CMS::traverse_face(&self, face.clone(), transit_strips);
                }
            }
            FaceType::LeafFace => {
                for strip in face.borrow().get_strips().iter() {
                    if strip.get_skip() {
                        assert!(strip.get_data(0) == -1);
                        continue;
                    }
                    transit_strips.push(strip.clone());
                }
            }
            FaceType::TransitFace => assert!(false),
        }
    }

    pub fn trace_comonent(&mut self) {
        let cells = self.octree.get_cells();
        for i in 0..MAX_OCTREE_RES {
            for cell in cells.iter() {
                if cell.borrow().get_cell_type() == &CellType::Leaf {
                    let mut cell_strips = Vec::new();
                    let mut transit_segs = Vec::new();
                    let mut components = Vec::new();

                    CMS::collect_strips(&self, cell.clone(), &mut cell_strips, &mut transit_segs);

                    loop {
                        if cell_strips.len() == 0 {
                            break;
                        }

                        CMS::link_strips(
                            &self,
                            &mut components,
                            &mut cell_strips,
                            &mut transit_segs,
                        );
                        cell.borrow_mut()
                            .get_componnets_mut()
                            .push(components.clone());
                        components.clear();
                    }
                }
            }
        }
    }

    pub fn collect_strips(
        self: &Self,
        cell: Rc<RefCell<Cell>>,
        cell_strips: &mut Vec<Strip>,
        transit_segs: &mut Vec<Vec<i8>>,
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
                        if strip.get_data(0) != -1 {
                            cell_strips.push(strip.clone());
                        }
                    }
                }
                FaceType::TransitFace => {
                    let twin = face_borrow.get_twin();
                    if twin.is_none() {
                        break;
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
                        if strip.get_data(0) != -1 {
                            transit_segs.push(
                                twin.as_ref().unwrap().borrow().get_transit_segs()[i].clone(),
                            );
                            cell_strips.push(strip.clone());
                        }
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
        transit_segs: &mut Vec<Vec<i8>>,
    ) {
        assert!(components.len() == 0);

        let mut added_in_iteration;
        let mut backwards;

        components.push(cell_strips[0].get_data(0) as usize);

        loop {
            added_in_iteration = 0;

            for strip in cell_strips.iter() {
                let s_data0 = strip.get_data(0);
                let s_data1 = strip.get_data(1);

                match components.last() {
                    Some(v) if *v == s_data0 as usize => {
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
                        }

                        if transit == false {
                            components.push(s_data1 as usize);
                            added_in_iteration += 1;
                        }
                    }
                    Some(v) if *v == s_data1 as usize => {
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
                        }

                        if transit == false {
                            components.push(s_data0 as usize);
                            added_in_iteration += 1;
                        }
                    }
                    _ => {}
                }
                // strips.remove(i);
            }

            if components.first() == components.last() {
                components.remove(0);
            }

            for comp in components.iter() {
                assert!(comp < &self.vertices.len());
            }

            if added_in_iteration <= 0 {
                break;
            }
        }

        loop {
            added_in_iteration = 0;

            for strip in cell_strips.iter() {
                let s_data0 = strip.get_data(0);
                let s_data1 = strip.get_data(1);

                match components.first() {
                    Some(v) if *v == s_data0 as usize => {
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
                        }

                        if transit == false {
                            components.insert(0, s_data1 as usize);
                            added_in_iteration += 1;
                        }
                    }
                    Some(v) if *v == s_data1 as usize => {
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
                        }

                        if transit == false {
                            components.insert(0, s_data0 as usize);
                            added_in_iteration += 1;
                        }
                    }
                    _ => {
                        continue;
                    }
                }
                // strips.remove(i);

                if components.first() == components.last() {
                    components.remove(0);
                }

                for comp in components.iter() {
                    assert!(comp < &self.vertices.len());
                }

                if added_in_iteration <= 0 {
                    break;
                }
            }

            assert!(components.first() != components.last());
            assert!(components.len() >= 3);
        }
    }

    fn insert_data_from_twin(
        components: &mut Vec<usize>,
        transit_segs: &Vec<Vec<i8>>,
        strip: &Strip,
        transit: &mut bool,
        added_in_iteration: &mut i32,
        backwards: &bool,
    ) {
        for seg in transit_segs.iter() {
            if CMS::compare_strip_to_seg(strip, seg) {
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
            }
        }
    }

    fn compare_strip_to_seg(strip: &Strip, seg: &Vec<i8>) -> bool {
        let s0 = strip.get_data(0);
        let s1 = strip.get_data(1);

        (seg.first().unwrap() == &s0 && seg.last().unwrap() == &s1)
            || (seg.first().unwrap() == &s1 && seg.last().unwrap() == &s0)
    }
}

impl CMS {
    pub fn tessellation_traversal(&mut self, cell: Rc<RefCell<Cell>>, mesh: &mut Mesh) {
        match cell.borrow().get_cell_type() {
            CellType::Branch => {
                for subcell_index in SubCellIndex::iter() {
                    assert!(cell.borrow().get_child(subcell_index).is_some());
                    if let Some(cell) = cell.borrow().get_child(subcell_index) {
                        self.tessellation_traversal(cell, mesh);
                    }
                }
            }
            CellType::Leaf => {
                if self.desired_cells.len() > 0 {
                    if self.desired_cells.contains(cell.borrow().get_id()) == false {
                        return;
                    }
                }

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
            .get_value(position.x, position.y + dimensions.x, position.z);

        let dz = self
            .sample_fn
            .get_value(position.x, position.y, position.z + dimensions.x);

        *normal = Vector3::new(dx - value, dy - value, dz - value);

        normal.normalize_mut();
    }

    pub fn make_tri(mesh: &mut Mesh, component: &Vec<usize>) {
        for i in 0..3 {
            mesh.get_indices_mut().push(component[i] as u32);
        }
    }

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
