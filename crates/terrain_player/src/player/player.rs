use crate::{
    shapes::{points::mesh::PointsMesh, triangles::mesh::TrianglesMesh},
};
use std::num::NonZeroU64;
use std::ops::Not;

use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{InputAction, Line, Point, Triangle};

use super::geometry_data::{AllGeometryData, OrderIdType};

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
            if terrain_chunk_coords.contains(&terrain_chunk_coord).not() {
                return false;
            }
        }

        true
    }
}

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
    let _action_state = input_query.single();

    let mut count = 0;
    loop {
        // if action_state.pressed(InputAction::NextOrder) {
        //     info!("next order");

        let mut player_order = None;
        loop {
            if player.current_order_id >= player.max_order_id {
                break;
            }

            player_order =
                all_geometry_data.get_geometry_data_order(player.current_order_id, &player_filter);
            player.current_order_id = player.current_order_id.checked_add(1).unwrap();

            assert!(player_order.is_some());
            if player_order.is_some() {
                break;
            }
        }

        if player_order.is_none() {
            break;
        }

        if let Some((terrain_chunk_coord, order)) = player_order {
            let geometry_data = all_geometry_data
                .get_geometry_data(terrain_chunk_coord)
                .unwrap();
            match order.geometry_data_order_type {
                super::geometry_data::GeometryDataOrderType::Vertex(index) => {
                    for (mesh, coord) in point_query.iter() {
                        if coord == terrain_chunk_coord {
                            if let Some(mesh) = meshes.get_mut(mesh) {
                                let vertices = geometry_data.vertices.get(index).unwrap();
                                PointsMesh::add_point(mesh, vertices);
                            }
                        }
                    }
                }
                super::geometry_data::GeometryDataOrderType::Line(_indices) => {
                    assert!(false);
                }
                super::geometry_data::GeometryDataOrderType::Triangle(indices) => {
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
        // }

        count += 1;
        if count > 100 {
            break;
        }
    }
}

pub fn pre_order() {}
