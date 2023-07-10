use bevy::prelude::Vec3;

#[derive(Debug)]
pub struct Edge {
    block: Vec3,
    dir: i32,

    empty: bool,
    vertex_index: i32,
}

impl Edge {
    pub fn new() -> Self {
        Self {
            block: Vec3::new(0.0, 0.0, 0.0),
            dir: -1,
            empty: true,
            vertex_index: -1,
        }
    }
}

impl Edge {
    pub fn set_block(&mut self, block: Vec3) {
        self.block = block;
    }

    pub fn set_dir(&mut self, dir: i32) {
        self.dir = dir;
    }

    pub fn set_empty(&mut self, empty: bool) {
        self.empty = empty;
    }

    pub fn set_vertex_index(&mut self, vertex_index: i32) {
        self.vertex_index = vertex_index;
    }

    pub fn get_block(&self) -> Vec3 {
        self.block
    }

    pub fn get_dir(&self) -> i32 {
        self.dir
    }

    pub fn get_empty(&self) -> bool {
        self.empty
    }

    pub fn get_vertex_index(&self) -> i32 {
        self.vertex_index
    }
}
