use crate::terrain::isosurface::dc::Cell;



#[derive(Debug)]
pub struct SeamOctree {

    #[allow(dead_code)]
    cells: Vec<Cell>, 
    #[allow(dead_code)]
    deep: u32,
}

impl SeamOctree {

    #[allow(dead_code)]
    pub fn new(cells: Vec<Cell>, deep: u32) -> Self {
        Self {
            cells,
            deep,
        }
    }

    #[allow(dead_code)]
    pub fn build(&self) {

    }
}