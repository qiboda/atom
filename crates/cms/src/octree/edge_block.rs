use nalgebra::Vector3;

use super::tables::Direction;

#[derive(Default, Clone, Debug)]
pub struct EdgeBlock {
    empty: bool,
    edge_index: Vector3<Option<usize>>,
}

impl EdgeBlock {
    pub fn new() -> Self {
        EdgeBlock {
            empty: true,
            edge_index: Vector3::new(None, None, None),
        }
    }
}

impl EdgeBlock {
    pub fn set_edge_index(&mut self, x: usize, y: usize, z: usize) {
        self.edge_index = Vector3::new(Some(x), Some(y), Some(z));
        self.empty = false;
    }

    pub fn set_dir_edge_index(&mut self, dir: Direction, edge_index: usize) {
        match dir {
            Direction::XAxis => self.edge_index.x = Some(edge_index),
            Direction::YAxis => self.edge_index.y = Some(edge_index),
            Direction::ZAxis => self.edge_index.z = Some(edge_index),
        }
        self.empty = false;
    }

    pub fn get_edge_index(&self) -> &Vector3<Option<usize>> {
        &self.edge_index
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn set_empty(&mut self) {
        self.empty = false;
        self.edge_index = Vector3::new(None, None, None);
    }
}
