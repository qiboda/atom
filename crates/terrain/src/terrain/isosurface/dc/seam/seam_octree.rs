use crate::terrain::isosurface::dc::Cell;



#[derive(Debug)]
pub struct SeamOctree {

    cells: Vec<Cell>, 
    deep: u32,
}

impl SeamOctree {

    pub fn new(cells: Vec<Cell>, deep: u32) -> Self {
        Self {
            cells,
            deep,
        }
    }

    pub fn build(&self) {

    }
}