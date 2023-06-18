// pub enum OctreeNodeType {
//     Branch,
//     Leaf,
// }
//
// pub struct OctreeNode<T> {
//     pub node_type: OctreeNodeType,
//     pub children: Option<[OctreeNode<T>; 8]>,
//
//     pub data: T,
// }
//
// pub struct Octree<T> {
//     root: OctreeNode<T>,
// }
//
// impl<T> Octree<T>
// where
//     T: Default,
// {
//     pub fn new() -> Self {
//         let root = OctreeNode {
//             node_type: OctreeNodeType::Branch,
//             children: Some([OctreeNode::<T>; 8]),
//             data: T::default(),
//         };
//
//         Self { root }
//     }
// }
