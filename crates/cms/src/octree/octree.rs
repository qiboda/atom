use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bevy::prelude::*;
use nalgebra::Vector3;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    address::Address,
    iso_surface::IsoSurface,
    octree::tables::{EDGE_DIRECTION, EDGE_VERTICES},
    sample::sample_range_3d::SampleRange3D,
    COMPLEX_SURFACE_THRESHOLD, MAX_OCTREE_RES, MIN_OCTREE_RES,
};

use super::{
    cell::{Cell, CellType},
    face::{Face, FaceType},
    tables::{
        EdgeIndex, FaceIndex, SubCellIndex, VertexPoint, FACE_DIRECTION, FACE_RELATIONSHIP_TABLE,
        FACE_TWIN_TABLE, NEIGHBOUR_ADDRESS_TABLE, SUB_FACE_TABLE,
    },
};

#[derive(Debug)]
pub struct Octree {
    root: Option<Rc<RefCell<Cell>>>,

    cells: Vec<Rc<RefCell<Cell>>>,

    leaf_cells: Vec<Rc<RefCell<Cell>>>,

    cell_addresses: HashMap<usize, Rc<RefCell<Cell>>>,

    faces: Vec<Rc<RefCell<Face>>>,

    samples_size: Vector3<usize>,

    sample_data: SampleRange3D<f32>,

    /// one cell location size
    offsets: Vector3<f32>,

    func: Rc<dyn IsoSurface>,
}

impl Octree {
    pub fn new(
        samples_size: Vector3<usize>,
        sample_data: SampleRange3D<f32>,
        offsets: Vector3<f32>,
        func: Rc<dyn IsoSurface>,
    ) -> Self {
        Octree {
            samples_size,
            sample_data,
            offsets,
            func,
            root: None,
            cells: vec![],
            leaf_cells: vec![],
            cell_addresses: HashMap::new(),
            faces: vec![],
        }
    }
}

impl Octree {
    pub fn get_root(&self) -> Option<Rc<RefCell<Cell>>> {
        self.root.clone()
    }

    pub fn get_cells(&self) -> &Vec<Rc<RefCell<Cell>>> {
        &self.cells
    }

    pub fn get_cell(&self, cell_id: usize) -> Option<Rc<RefCell<Cell>>> {
        self.cells.get(cell_id).cloned()
    }
}

impl Octree {
    pub fn build_octree(&mut self) {
        self.make_structure();

        self.populate_half_faces();

        self.set_face_relationship();

        self.mark_transitional_faces();
    }

    fn make_structure(&mut self) {
        let c000 = Vector3::new(0, 0, 0);

        self.root = Some(Rc::new(RefCell::new(Cell::new(
            0,
            CellType::Branch,
            None,
            0,
            c000,
            self.samples_size - Vector3::new(1, 1, 1),
            None,
        ))));

        self.acquire_cell_info(self.root.clone().unwrap());
        self.subdivide_cell(self.root.clone().unwrap());
    }

    fn acquire_cell_info(&mut self, cell: Rc<RefCell<Cell>>) {
        let mut pt_indices = [Vector3::new(0, 0, 0); VertexPoint::COUNT];

        {
            let cell_borrow = cell.borrow();
            let c000 = cell_borrow.get_c000();
            let offsets_size = cell_borrow.get_offsets_size();

            pt_indices[0] = Vector3::new(c000.x, c000.y, c000.z);
            pt_indices[1] = Vector3::new(c000.x, c000.y, c000.z + offsets_size.z);
            pt_indices[2] = Vector3::new(c000.x, c000.y + offsets_size.y, c000.z);
            pt_indices[3] = Vector3::new(c000.x, c000.y + offsets_size.y, c000.z + offsets_size.z);
            pt_indices[4] = Vector3::new(c000.x + offsets_size.x, c000.y, c000.z);
            pt_indices[5] = Vector3::new(c000.x + offsets_size.x, c000.y, c000.z + offsets_size.z);
            pt_indices[6] = Vector3::new(c000.x + offsets_size.x, c000.y + offsets_size.y, c000.z);
            pt_indices[7] = Vector3::new(
                c000.x + offsets_size.x,
                c000.y + offsets_size.y,
                c000.z + offsets_size.z,
            );

            // 排除右边缘
            for pt_index in pt_indices.iter_mut() {
                pt_index.x = pt_index.x.clamp(0, self.samples_size.x - 1);
                pt_index.y = pt_index.y.clamp(0, self.samples_size.y - 1);
                pt_index.z = pt_index.z.clamp(0, self.samples_size.z - 1);
            }
        }

        cell.borrow_mut().set_corner_sample_index(pt_indices);
    }

    fn subdivide_cell(&mut self, parent_cell: Rc<RefCell<Cell>>) {
        let this_level = parent_cell.borrow().get_cur_subdiv_level() + 1;

        info!("subdivide_cell: this level: {}", this_level);

        let mut sample_size = Vector3::new(0, 0, 0);

        sample_size[0] = (self.samples_size[0] - 1) >> this_level;
        sample_size[1] = (self.samples_size[1] - 1) >> this_level;
        sample_size[2] = (self.samples_size[2] - 1) >> this_level;

        info!("subdivide_cell: sample size: {}", sample_size);

        let parent_c000 = *parent_cell.borrow().get_c000();

        for (i, subcell_index) in SubCellIndex::iter().enumerate() {
            let c000 = Vector3::new(
                parent_c000.x + sample_size.x * ((i >> 2) & 1),
                parent_c000.y + sample_size.y * ((i >> 1) & 1),
                parent_c000.z + sample_size.z * (i & 1),
            );

            let cell = Rc::new(RefCell::new(Cell::new(
                self.cells.len(),
                CellType::Branch,
                Some(Rc::clone(&parent_cell)),
                this_level,
                c000,
                sample_size,
                Some(subcell_index),
            )));

            self.acquire_cell_info(cell.clone());
            self.cells.push(cell.clone());

            parent_cell
                .borrow_mut()
                .set_child(i as usize, Some(Rc::clone(&cell)));

            // info!(
            //     "subdivide_cell: cell: {:?}",
            //     cell.borrow().get_corner_sample_index()
            // );

            match this_level {
                _ if (0..MIN_OCTREE_RES).contains(&(this_level as usize)) => {
                    self.subdivide_cell(Rc::clone(&cell));
                }
                _ if (MIN_OCTREE_RES..MAX_OCTREE_RES).contains(&(this_level as usize)) => {
                    if self.check_for_subdivision(cell.clone()) {
                        self.subdivide_cell(cell.clone());
                    } else {
                        // todo: 如此，如果不是在表面，就会忽略cell，这是否正确？
                        //
                        let surface = self.check_for_surface(cell.clone());
                        // info!("{this_level}:{i}: check_for_surface: {}", surface);
                        if surface {
                            // info!("{this_level}:{i}: set leaf");
                            cell.borrow_mut().set_cell_type(CellType::Leaf);
                            self.leaf_cells.push(cell.clone());
                        }
                    }
                }
                _ => {
                    // todo: 如此，如果不是在表面，就会忽略cell，这是否正确？
                    if self.check_for_surface(cell.clone()) {
                        // info!("{this_level}:{i}: set leaf");
                        cell.borrow_mut().set_cell_type(CellType::Leaf);
                        self.leaf_cells.push(cell.clone());
                    }
                }
            }

            self.cell_addresses
                .insert(cell.borrow().get_address().get_formatted(), cell.clone());
        }
    }

    // 检查是否在表面
    fn check_for_surface(&mut self, cell: Rc<RefCell<Cell>>) -> bool {
        let pos_in_parent = *cell.borrow().get_corner_sample_index();

        // 8个顶点中有几个在内部
        let mut inside = 0;
        for i in 0..8 {
            if self.sample_data.get_value(
                pos_in_parent[i].x,
                pos_in_parent[i].y,
                pos_in_parent[i].z,
            ) < 0.0
            {
                inside += 1;
            }
        }

        // info!("check_for_surface: inside: {}", inside);

        inside != 0 && inside != 8
    }

    fn check_for_subdivision(&self, cell: Rc<RefCell<Cell>>) -> bool {
        let edge_ambiguity = self.check_for_edge_ambiguity(cell.clone());
        let complex_surface = self.check_for_complex_surface(cell.clone());
        // info!(
        //     "check_for_subdivision: {}, {}",
        //     edge_ambiguity, complex_surface
        // );
        edge_ambiguity || complex_surface
    }

    /// 检测是否(坐标位置)平坦
    fn check_for_edge_ambiguity(&self, cell: Rc<RefCell<Cell>>) -> bool {
        let mut edge_ambiguity = false;

        let cell = cell.borrow();
        let points = cell.get_corner_sample_index();

        for (i, _edge_index) in EdgeIndex::iter().enumerate() {
            let vertex_index_0 = EDGE_VERTICES[i][0] as usize;
            let vertex_index_1 = EDGE_VERTICES[i][1] as usize;

            let edge_direction = EDGE_DIRECTION[i];

            // info!("edge_direction: {:?}", edge_direction);

            // left coord
            let point_0 = points[vertex_index_0];
            // right coord
            let point_1 = points[vertex_index_1];

            // info!("point0: {:?} point1: {:?}", point_0, point_1);

            // max right index
            let last_index = self
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

                assert!(self.sample_data.get_data_index(index.x, index.y, index.z) <= last_index);

                // if the sign of the value at the previous point is different from the sign of the value at the current point,
                // then there is an edge ambiguity
                if self
                    .sample_data
                    .get_value(prev_point.x, prev_point.y, prev_point.z)
                    * self.sample_data.get_value(index.x, index.y, index.z)
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
    fn check_for_complex_surface(&self, cell: Rc<RefCell<Cell>>) -> bool {
        let mut complex_surface = false;

        let cell = cell.borrow();
        let points = cell.get_corner_sample_index();

        'outer: for i in 0..7 {
            let point_0 = points[i];

            let mut normal_point_0 = Default::default();
            self.find_gradient(&mut normal_point_0, &point_0);

            for j in 1..8 {
                let point_1 = points[j];

                let mut normal_point_1 = Default::default();

                self.find_gradient(&mut normal_point_1, &point_1);

                if normal_point_0.dot(&normal_point_1) < COMPLEX_SURFACE_THRESHOLD {
                    complex_surface = true;
                    break 'outer;
                }
            }
        }

        complex_surface
    }

    fn find_gradient(&self, gradient: &mut Vector3<f32>, point: &Vector3<usize>) {
        let pos = self.sample_data.get_pos(point.x, point.y, point.z);

        let mut dimensions = Vector3::new(0.0, 0.0, 0.0);

        // why use half offset?
        for i in 0..3 {
            dimensions[i] = self.offsets[i] / 2.0;
        }

        let dx = self.func.get_value(pos.x + dimensions.x, pos.y, pos.z);
        let dy = self.func.get_value(pos.x, pos.y + dimensions.y, pos.z);
        let dz = self.func.get_value(pos.x, pos.y, pos.z + dimensions.z);
        let val = self.sample_data.get_value(point.x, point.y, point.z);

        *gradient = Vector3::new(dx - val, dy - val, dz - val);
        gradient.normalize_mut();
    }

    fn populate_half_faces(&mut self) {
        for cell in &self.cells {
            let mut contact_cell_address = [
                Address::new(),
                Address::new(),
                Address::new(),
                Address::new(),
                Address::new(),
                Address::new(),
            ];

            let mut temp_neightbour_address = [vec![], vec![], vec![], vec![], vec![], vec![]];
            for (i, _) in FaceIndex::iter().enumerate() {
                temp_neightbour_address[i].resize(MAX_OCTREE_RES, None);
            }

            for (i, face_index) in FaceIndex::iter().enumerate() {
                let mut same_parent = false;
                for j in (0..MAX_OCTREE_RES).rev() {
                    if same_parent {
                        // 只是为了减少计算地址。因为有相同的父级，更上层的地址肯定是一样的。
                        temp_neightbour_address[i][j] = cell.borrow().get_address().get_raw()[j];
                    } else {
                        // 得到对应层级的在父级的位置。
                        let value = cell.borrow().get_address().get_raw()[j];

                        let axis = FACE_DIRECTION[i];

                        match value {
                            Some(v) => {
                                temp_neightbour_address[i][j] =
                                    Some(NEIGHBOUR_ADDRESS_TABLE[axis as usize][v as usize]);
                            }
                            None => {
                                temp_neightbour_address[i][j] = None;
                            }
                        }

                        if let (Some(v), Some(t)) = (value, temp_neightbour_address[i][j]) {
                            match face_index {
                                FaceIndex::Back => {
                                    if v as usize > t as usize {
                                        same_parent = true;
                                    }
                                }
                                FaceIndex::Front => {
                                    if (v as usize) < (t as usize) {
                                        same_parent = true;
                                    }
                                }
                                FaceIndex::Bottom => {
                                    if v as usize > t as usize {
                                        same_parent = true;
                                    }
                                }
                                FaceIndex::Top => {
                                    if (v as usize) < (t as usize) {
                                        same_parent = true;
                                    }
                                }
                                FaceIndex::Left => {
                                    if v as usize > t as usize {
                                        same_parent = true;
                                    }
                                }
                                FaceIndex::Right => {
                                    if (v as usize) < (t as usize) {
                                        same_parent = true;
                                    }
                                }
                            }
                        }
                    }
                }

                contact_cell_address[i].populate_address(&temp_neightbour_address[i]);
            }

            for (i, face_index) in FaceIndex::iter().enumerate() {
                let address_key = contact_cell_address[i].get_formatted();

                let contact_cell = self.cell_addresses.get(&address_key);

                if contact_cell.is_some() {
                    let mut it = FaceIndex::iter();
                    if i % 2 == 0 {
                        cell.borrow_mut().set_neighbor(
                            it.nth(i + 1).unwrap(),
                            Some(contact_cell.unwrap().clone()),
                        );
                    } else {
                        cell.borrow_mut().set_neighbor(
                            it.nth(i - 1).unwrap(),
                            Some(contact_cell.unwrap().clone()),
                        );
                    }

                    self.set_face_twins(contact_cell.unwrap().clone(), cell.clone(), face_index);
                }
            }
        }
    }

    fn set_face_twins(
        &self,
        cell_1: Rc<RefCell<Cell>>,
        cell_2: Rc<RefCell<Cell>>,
        face_index: FaceIndex,
    ) {
        let val_2 = FACE_TWIN_TABLE[face_index as usize][0];
        let val_1 = FACE_TWIN_TABLE[face_index as usize][1];

        cell_2
            .borrow_mut()
            .get_face(val_2)
            .borrow_mut()
            .set_twin(cell_1.borrow().get_face(val_1).clone());

        cell_1
            .borrow_mut()
            .get_face(val_1)
            .borrow_mut()
            .set_twin(cell_2.borrow().get_face(val_2).clone());

        let cell_2 = cell_2.borrow();
        let face = cell_2.get_face(face_index);
        let cell_2_face = face.borrow();
        let id = cell_2_face.get_face_index();

        let cell_2_face_twin = cell_2_face.get_twin();
        cell_2_face_twin.clone().map(|x| {
            let cell_2_face_twin = x.borrow();
            cell_2_face_twin.get_twin().clone().map(|x| {
                let id_2 = x.borrow().get_face_index();
                assert!(id == id_2);
            });
        });
    }

    fn set_face_relationship(&self) {
        for cell in &self.cells {
            if let &Some(pos_in_parent) = cell.borrow().get_pos_in_parent() {
                for side in 0..3 {
                    let face_index = FACE_RELATIONSHIP_TABLE[pos_in_parent as usize][side];
                    let sub_face_index = SUB_FACE_TABLE[pos_in_parent as usize][side];

                    let cell_b = cell.borrow();
                    let parent = cell_b.get_parent().as_ref().unwrap();

                    let face = parent.borrow().get_face(face_index);

                    cell.borrow()
                        .get_face(face_index)
                        .borrow_mut()
                        .set_parent(face);

                    parent
                        .borrow()
                        .get_face(face_index)
                        .borrow_mut()
                        .set_child(sub_face_index, cell.borrow().get_face(face_index).clone());
                }

                if *cell.borrow().get_cell_type() == CellType::Leaf {
                    for face_index in FaceIndex::iter() {
                        cell.borrow()
                            .get_face(face_index)
                            .borrow_mut()
                            .set_face_type(FaceType::LeafFace);
                    }
                }
            }
        }
    }

    fn mark_transitional_faces(&self) {
        for leaf_cell in &self.leaf_cells {
            assert!(leaf_cell.borrow().get_cell_type() == &CellType::Leaf);

            for face_index in FaceIndex::iter() {
                let cell = leaf_cell.borrow();
                let face = cell.get_face(face_index);

                let mut setface = false;

                {
                    let face_b = face.borrow_mut();
                    assert!(face_b.get_face_type() == &FaceType::LeafFace);

                    if let Some(twin) = face_b.get_twin() {
                        if let Some(children) = twin.borrow().get_children() {
                            if children.len() > 0 {
                                setface = true;
                            }
                        }
                    }
                }

                if setface {
                    let mut face_bm = face.borrow_mut();
                    face_bm.set_face_type(FaceType::TransitFace);
                }
            }
        }
    }
}
