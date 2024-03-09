pub mod geometry_data;
pub mod order;

use crate::shapes::{points::mesh::PointsMesh, triangles::mesh::TrianglesMesh};
use std::num::NonZeroU64;
use std::ops::Not;

use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{InputAction, Line, Point, Triangle};

use geometry_data::{AllGeometryData, OrderIdType};

#[derive(Resource, Debug)]
pub struct Player {
    pub current_order_id: OrderIdType,
    pub max_order_id: OrderIdType,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            current_order_id: OrderIdType::new(1).unwrap(),
            max_order_id: OrderIdType::new(1).unwrap(),
        }
    }
}

// 过滤方式设置,修改之后，重新播放到指定位置(order id)。
#[derive(Resource, Default, Debug)]
pub struct PlayerFilter {
    pub thread_ids: Option<Vec<NonZeroU64>>,
    pub terrain_chunk_coords: Option<Vec<TerrainChunkCoord>>,
}

impl PlayerFilter {
    pub fn filter(&self, thread_id: NonZeroU64, terrain_chunk_coord: &TerrainChunkCoord) -> bool {
        if let Some(thread_ids) = &self.thread_ids {
            if thread_ids.contains(&thread_id).not() {
                return false;
            }
        }

        if let Some(terrain_chunk_coords) = &self.terrain_chunk_coords {
            if terrain_chunk_coords.contains(terrain_chunk_coord).not() {
                return false;
            }
        }

        true
    }
}

#[allow(clippy::too_many_arguments)]
pub fn next_order(
    input_query: Query<&ActionState<InputAction>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut player: ResMut<Player>,
    player_filter: Res<PlayerFilter>,
    point_query: Query<(&Handle<Mesh>, &TerrainChunkCoord), With<Point>>,
    _line_query: Query<(&Handle<Mesh>, &TerrainChunkCoord), With<Line>>,
    triangle_query: Query<(&Handle<Mesh>, &TerrainChunkCoord), With<Triangle>>,
    all_geometry_data: Res<AllGeometryData>,
) {
    let action_state = input_query.single();

    let mut count = 0;
    if action_state.pressed(&InputAction::NextOrder) {
        count = 1;
        info!("next order");
    }

    if action_state.pressed(&InputAction::NextHundredOrder) {
        count = 100;
        info!("next hundred order");
    }

    for _i in 0..count {
        let mut player_order = None;
        loop {
            if player.current_order_id >= player.max_order_id {
                break;
            }

            player_order =
                all_geometry_data.get_geometry_data_order(player.current_order_id, &player_filter);
            player.current_order_id = player.current_order_id.checked_add(1).unwrap();

            info!("current order id: {}", player.current_order_id);

            assert!(player_order.is_some());
            if player_order.is_some() {
                break;
            }
        }

        if player_order.is_none() {
            return;
        }

        if let Some((terrain_chunk_coord, order)) = player_order {
            let geometry_data = all_geometry_data
                .get_geometry_data(terrain_chunk_coord)
                .unwrap();
            match order.geometry_data_order_type {
                geometry_data::GeometryDataOrderType::Vertex(index) => {
                    for (mesh, coord) in point_query.iter() {
                        if coord == terrain_chunk_coord {
                            if let Some(mesh) = meshes.get_mut(mesh) {
                                let vertices = geometry_data.vertices.get(index).unwrap();
                                PointsMesh::add_point(mesh, vertices);
                            }
                        }
                    }
                }
                geometry_data::GeometryDataOrderType::Line(_indices) => {
                    unreachable!();
                }
                geometry_data::GeometryDataOrderType::Triangle(indices) => {
                    for (mesh, coord) in triangle_query.iter() {
                        if coord == terrain_chunk_coord {
                            if let Some(mesh) = meshes.get_mut(mesh) {
                                TrianglesMesh::add_all_vertices(mesh, &geometry_data.vertices);
                                TrianglesMesh::add_triangle_indices(mesh, &indices);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn pre_order(
    input_query: Query<&ActionState<InputAction>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut player: ResMut<Player>,
    player_filter: Res<PlayerFilter>,
    point_query: Query<(&Handle<Mesh>, &TerrainChunkCoord), With<Point>>,
    _line_query: Query<(&Handle<Mesh>, &TerrainChunkCoord), With<Line>>,
    triangle_query: Query<(&Handle<Mesh>, &TerrainChunkCoord), With<Triangle>>,
    all_geometry_data: Res<AllGeometryData>,
) {
    let mut count = 0;

    let action_state = input_query.single();
    if action_state.pressed(&InputAction::PreOrder) {
        count = 1;
        info!("Pre order");
    }
    if action_state.pressed(&InputAction::PreHundredOrder) {
        count = 100;
        info!("Pre Hundred order");
    }

    for _i in 0..count {
        let mut player_order = None;
        loop {
            if player.current_order_id.get() == 1 {
                break;
            }

            player_order =
                all_geometry_data.get_geometry_data_order(player.current_order_id, &player_filter);
            player.current_order_id =
                NonZeroU64::new(player.current_order_id.get().checked_sub(1u64).unwrap()).unwrap();

            info!("current order id: {}", player.current_order_id);

            assert!(player_order.is_some());
            if player_order.is_some() {
                break;
            }
        }

        if player_order.is_none() {
            return;
        }

        if let Some((terrain_chunk_coord, order)) = player_order {
            match order.geometry_data_order_type {
                geometry_data::GeometryDataOrderType::Vertex(index) => {
                    for (mesh, coord) in point_query.iter() {
                        if coord == terrain_chunk_coord {
                            if let Some(mesh) = meshes.get_mut(mesh) {
                                PointsMesh::remove_point_at_index(mesh, index);
                            }
                        }
                    }
                }
                geometry_data::GeometryDataOrderType::Line(_indices) => {
                    unreachable!()
                }
                geometry_data::GeometryDataOrderType::Triangle(_indices) => {
                    for (mesh, coord) in triangle_query.iter() {
                        if coord == terrain_chunk_coord {
                            if let Some(mesh) = meshes.get_mut(mesh) {
                                TrianglesMesh::remove_last_triangle_indices(mesh);
                            }
                        }
                    }
                }
            }
        }
    }
}
