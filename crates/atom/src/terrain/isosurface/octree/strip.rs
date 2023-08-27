

use super::tables::{Face2DEdge};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strip {
    b_loop: bool,

    // todo: 可能没有用，删除。
    b_skip: bool,

    edge: [Option<Face2DEdge>; 2],
    /// every edge CMS::vertices index.
    vertex_index: [Option<u32>; 2],
}

impl Default for Strip {
    fn default() -> Self {
        Self {
            b_skip: true,
            b_loop: false,
            edge: [None; 2],
            vertex_index: [None; 2],
        }
    }
}

impl Strip {
    pub fn new(edge0: Option<Face2DEdge>, edge1: Option<Face2DEdge>) -> Strip {
        Self {
            b_skip: true,
            b_loop: false,
            edge: [edge0, edge1],
            vertex_index: [None; 2],
        }
    }
}

impl Strip {
    pub fn get_edge(&self, index: usize) -> Option<Face2DEdge> {
        self.edge[index]
    }

    pub fn set_vertex_index(&mut self, index: usize, vertex_index: u32) {
        self.vertex_index[index] = Some(vertex_index);
    }

    pub fn get_vertex_index(&self, index: usize) -> Option<u32> {
        self.vertex_index[index]
    }

    pub fn get_vertex(&self) -> &[Option<u32>; 2] {
        &self.vertex_index
    }

    pub fn set_loop(&mut self, b_loop: bool) {
        self.b_loop = b_loop;
    }

    pub fn get_loop(&self) -> bool {
        self.b_loop
    }

    pub fn set_skip(&mut self, b_skip: bool) {
        self.b_skip = b_skip;
    }

    pub fn get_skip(&self) -> bool {
        self.b_skip
    }
}

impl Strip {
    pub fn change_back(&mut self, s: &Strip, i: usize) {
        self.edge[1] = s.edge[i];
        self.vertex_index[1] = s.vertex_index[i];
    }

    pub fn change_front(&mut self, s: &Strip, i: usize) {
        self.edge[0] = s.edge[i];
        self.vertex_index[0] = s.vertex_index[i];
    }
}
