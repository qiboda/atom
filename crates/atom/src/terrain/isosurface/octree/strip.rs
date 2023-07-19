use bevy::prelude::UVec3;

use super::tables::{EdgeDirection, Face2DEdge};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strip {
    b_loop: bool,

    edge: [Option<Face2DEdge>; 2],
    /// every edge CMS::vertices index.
    vertex_index: [Option<u32>; 2],
    /// every edge crossing point
    crossing_left_coord: [Option<UVec3>; 2],
    /// every edge direction
    edge_dir: [Option<EdgeDirection>; 2],
}

impl Default for Strip {
    fn default() -> Self {
        Self {
            b_loop: false,
            edge: [None; 2],
            vertex_index: [None; 2],
            crossing_left_coord: [None; 2],
            edge_dir: [None; 2],
        }
    }
}

impl Strip {
    pub fn new(edge0: Option<Face2DEdge>, edge1: Option<Face2DEdge>) -> Strip {
        Self {
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

    pub fn set_vertex_index(&mut self, index: usize, vertex_index: u32) {
        self.vertex_index[index] = Some(vertex_index);
    }

    pub fn get_vertex_index(&self, index: usize) -> Option<u32> {
        self.vertex_index[index]
    }

    pub fn get_vertex(&self) -> &[Option<u32>; 2] {
        &self.vertex_index
    }

    pub fn set_crossing_left_coord(&mut self, index: usize, block: UVec3) {
        self.crossing_left_coord[index] = Some(block);
    }

    pub fn get_crossing_left_coord(&self, index: usize) -> Option<UVec3> {
        self.crossing_left_coord[index]
    }

    pub fn set_edge_dir(&mut self, index: usize, dir: Option<EdgeDirection>) {
        self.edge_dir[index] = dir;
    }

    pub fn get_dir(&self, index: usize) -> Option<EdgeDirection> {
        self.edge_dir[index]
    }

    pub fn set_loop(&mut self, b_loop: bool) {
        self.b_loop = b_loop;
    }

    pub fn get_loop(&self) -> bool {
        self.b_loop
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
