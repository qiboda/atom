use nalgebra::Vector3;

use super::tables::{Direction, Face2DEdge};

#[derive(Clone, Debug)]
pub struct Strip {
    skip: bool,
    b_loop: bool,

    edge: [Option<Face2DEdge>; 2],
    /// every edge CMS::vertices index.
    vertex_index: [Option<usize>; 2],
    /// every edge crossing point
    crossing_left_coord: [Option<Vector3<usize>>; 2],
    /// every edge direction
    edge_dir: [Option<Direction>; 2],
}

impl Default for Strip {
    fn default() -> Self {
        Self {
            skip: false,
            b_loop: false,
            edge: [None; 2],
            vertex_index: [None; 2],
            crossing_left_coord: [None; 2],
            edge_dir: [None; 2],
        }
    }
}

impl Strip {
    pub fn new(skip: bool, edge0: Option<Face2DEdge>, edge1: Option<Face2DEdge>) -> Strip {
        Self {
            skip,
            b_loop: false,
            edge: [edge0, edge1],
            vertex_index: [None; 2],
            crossing_left_coord: [None; 2],
            edge_dir: [None; 2],
        }
    }
}

impl Strip {
    pub fn get_edge(&self, index: usize) -> Option<Face2DEdge> {
        self.edge[index]
    }

    pub fn set_vertex_index(&mut self, index: usize, vertex_index: usize) {
        self.vertex_index[index] = Some(vertex_index);
    }

    pub fn get_vertex_index(&self, index: usize) -> Option<usize> {
        self.vertex_index[index]
    }

    pub fn set_crossing_left_coord(&mut self, index: usize, block: Vector3<usize>) {
        self.crossing_left_coord[index] = Some(block);
    }

    pub fn get_crossing_left_coord(&self, index: usize) -> Option<Vector3<usize>> {
        self.crossing_left_coord[index]
    }

    pub fn set_edge_dir(&mut self, index: usize, dir: Option<Direction>) {
        self.edge_dir[index] = dir;
    }

    pub fn get_dir(&self, index: usize) -> Option<Direction> {
        self.edge_dir[index]
    }

    pub fn set_loop(&mut self, b_loop: bool) {
        self.b_loop = b_loop;
    }

    pub fn get_loop(&self) -> bool {
        self.b_loop
    }

    pub fn set_skip(&mut self, skip: bool) {
        self.skip = skip;
    }

    pub fn get_skip(&self) -> bool {
        self.skip
    }
}

impl Strip {
    pub fn change_back(&mut self, s: &Strip, i: usize) {
        self.edge[1] = s.edge[i];
        self.vertex_index[1] = s.vertex_index[i];
        self.edge_dir[1] = s.edge_dir[i];
        self.crossing_left_coord[1] = s.crossing_left_coord[i];
    }

    pub fn change_front(&mut self, s: &Strip, i: usize) {
        self.edge[0] = s.edge[i];
        self.vertex_index[0] = s.vertex_index[i];
        self.edge_dir[0] = s.edge_dir[i];
        self.crossing_left_coord[0] = s.crossing_left_coord[i];
    }
}
