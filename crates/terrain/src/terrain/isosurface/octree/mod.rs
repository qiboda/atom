pub mod address;
pub mod cell;
pub mod face;
pub mod octree;
pub mod vertex;
pub mod tables;
pub mod edge;

pub trait OctreeBranchPolicy {
    fn check_to_subdivision(&self) -> bool;
}
