use nalgebra::Vector3;

use super::tables::{Direction, Face2DEdge};

#[derive(Clone, Debug)]
pub struct Strip {
    skip: bool,
    b_loop: bool,

    edge: [Option<Face2DEdge>; 2],
    data: [i8; 2],

    block: [Vector3<usize>; 2],
    dir: [Option<Direction>; 2],
}

impl Default for Strip {
    fn default() -> Self {
        Self {
            skip: true,
            b_loop: false,
            edge: [None; 2],
            data: [-1; 2],
            block: [Vector3::new(0, 0, 0); 2],
            dir: [None; 2],
        }
    }
}

impl Strip {
    pub fn new(skip: bool, edge0: Option<Face2DEdge>, edge1: Option<Face2DEdge>) -> Strip {
        Self {
            skip,
            b_loop: false,
            edge: [edge0, edge1],
            data: [-1; 2],
            block: [Vector3::new(0, 0, 0); 2],
            dir: [None; 2],
        }
    }
}

impl Strip {
    pub fn get_edge(&self, index: usize) -> Option<Face2DEdge> {
        self.edge[index]
    }

    pub fn set_data(&mut self, index: usize, data: i8) {
        self.data[index] = data;
    }

    pub fn get_data(&self, index: usize) -> i8 {
        self.data[index]
    }

    pub fn set_block(&mut self, index: usize, block: Vector3<usize>) {
        self.block[index] = block;
    }

    pub fn get_block(&self, index: usize) -> Vector3<usize> {
        self.block[index]
    }

    pub fn set_dir(&mut self, index: usize, dir: Option<Direction>) {
        self.dir[index] = dir;
    }

    pub fn get_dir(&self, index: usize) -> Option<Direction> {
        self.dir[index]
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
        self.data[1] = s.data[i];
        self.dir[1] = s.dir[i];
        self.block[1] = s.block[i];
    }

    pub fn change_front(&mut self, s: &Strip, i: usize) {
        self.edge[0] = s.edge[i];
        self.data[0] = s.data[i];
        self.dir[0] = s.dir[i];
        self.block[0] = s.block[i];
    }
}
