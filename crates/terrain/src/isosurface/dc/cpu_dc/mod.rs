// /// 获取缝隙的位置，

// pub mod dc_gizmos;
// pub mod dual_contouring;
// pub mod main_mesh;
// pub mod octree;
// pub mod seam_mesh;

// use bevy::prelude::*;
// use dc_gizmos::DcGizmosPlugin;

// #[derive(Debug, Default, Reflect)]
// pub struct DualContouringPlugin;

// impl Plugin for DualContouringPlugin {
//     fn build(&self, app: &mut App) {
//         app.configure_sets(Update)
//             .add_plugins(DcGizmosPlugin)
//             .add_systems(
//                 Update,
//                 (seam_mesh::construct_octree, seam_mesh::dual_contouring).chain(),
//             );
//     }
// }
pub mod cpu_seam;
