use std::num::NonZeroU64;

use crate::player::order::Orders;
use bevy::{prelude::*, utils::HashMap};
use terrain_core::chunk::coords::TerrainChunkCoord;
use terrain_player_client::order::OrderType;

use super::player::{Player, PlayerFilter};

// todo: 分为每chunk顶点和索引。
#[derive(Debug, Default, Resource)]
pub struct AllGeometryData {
    pub geometry_data_map: HashMap<TerrainChunkCoord, GeometryData>,
}

impl AllGeometryData {
    pub fn get_geometry_data_order(
        &self,
        order_id: OrderIdType,
        player_filter: &Res<PlayerFilter>,
    ) -> Option<(&TerrainChunkCoord, &GeometryDataOrder)> {
        for (terrain_chunk_coord, geometry_data) in self.geometry_data_map.iter() {
            if let Some(order) = geometry_data.geometry_data_orders.get(&order_id) {
                assert!(player_filter.filter(order.thread_id, terrain_chunk_coord));
                if player_filter.filter(order.thread_id, terrain_chunk_coord) {
                    return Some((terrain_chunk_coord, order));
                }
            }
        }
        assert!(false, "order id: {} not found", order_id);
        None
    }

    pub fn get_geometry_data(
        &self,
        terrain_chunk_coord: &TerrainChunkCoord,
    ) -> Option<&GeometryData> {
        self.geometry_data_map.get(terrain_chunk_coord)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GeometryDataOrderType {
    Vertex(usize),
    Line([u32; 2]),
    Triangle([u32; 3]),
}

impl Default for GeometryDataOrderType {
    fn default() -> Self {
        Self::Vertex(0)
    }
}

pub type OrderIdType = NonZeroU64;

#[derive(Debug)]
pub struct GeometryDataOrder {
    pub order_id: OrderIdType,
    pub thread_id: NonZeroU64,
    pub geometry_data_order_type: GeometryDataOrderType,
}

impl Default for GeometryDataOrder {
    fn default() -> Self {
        Self {
            order_id: NonZeroU64::MAX,
            thread_id: NonZeroU64::MAX,
            geometry_data_order_type: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GeometryData {
    pub vertices: Vec<[f32; 3]>,

    pub geometry_data_orders: HashMap<OrderIdType, GeometryDataOrder>,
    pub terrain_chunk_coord: TerrainChunkCoord,
}

pub fn process_player_order(
    mut all_geometry_data: ResMut<AllGeometryData>,
    player_orders: Res<Orders>,
    mut player: ResMut<Player>,
) {
    for order in &player_orders.orders {
        let geometry_data = all_geometry_data
            .geometry_data_map
            .entry(order.get_terrain_chunk_coord())
            .or_default();

        geometry_data.terrain_chunk_coord = order.get_terrain_chunk_coord();

        if order.order_id == NonZeroU64::new(2835).unwrap() {
            info!("exist 2835");
        }

        player.max_order_id = player.max_order_id.max(order.order_id);

        match &order.fields.order_type {
            OrderType::Vertex(vertex_data) => {
                geometry_data.geometry_data_orders.insert(
                    order.order_id,
                    GeometryDataOrder {
                        order_id: order.order_id,
                        thread_id: order.thread_id,
                        geometry_data_order_type: GeometryDataOrderType::Vertex(
                            geometry_data.vertices.len(),
                        ),
                    },
                );
                geometry_data.vertices.push(vertex_data.location.to_array());
            }
            OrderType::Line(line_data) => {
                geometry_data.geometry_data_orders.insert(
                    order.order_id,
                    GeometryDataOrder {
                        order_id: order.order_id,
                        thread_id: order.thread_id,
                        geometry_data_order_type: GeometryDataOrderType::Line([
                            line_data.start_index as u32,
                            line_data.end_index as u32,
                        ]),
                    },
                );
            }
            OrderType::Triangle(triangle_data) => {
                geometry_data.geometry_data_orders.insert(
                    order.order_id,
                    GeometryDataOrder {
                        order_id: order.order_id,
                        thread_id: order.thread_id,
                        geometry_data_order_type: GeometryDataOrderType::Triangle([
                            triangle_data.vertex_index_0 as u32,
                            triangle_data.vertex_index_1 as u32,
                            triangle_data.vertex_index_2 as u32,
                        ]),
                    },
                );
            }
        }
    }

    info!("player max order id: {}", player.max_order_id);
    for (terrain_chunk_coord, geometry_data) in all_geometry_data.geometry_data_map.iter() {
        let mut vertex_order_counter = 0;
        let mut line_order_counter = 0;
        let mut triangle_order_counter = 0;
        let mut vertex_indices = vec![];
        let mut line_indices = vec![];
        let mut triangle_indices = vec![];
        for (_, order) in geometry_data.geometry_data_orders.iter() {
            match order.geometry_data_order_type {
                GeometryDataOrderType::Vertex(index) => {
                    vertex_order_counter += 1;
                    vertex_indices.push(index);
                }
                GeometryDataOrderType::Line(indices) => {
                    line_order_counter += 1;
                    line_indices.push(indices);
                }
                GeometryDataOrderType::Triangle(indices) => {
                    triangle_order_counter += 1;
                    triangle_indices.push(indices);
                }
            }
        }
        info!(
            "terrain_chunk_coord: {:?}, vertices: {}, orders: {}, vertex orders: {}, line orders: {}, triangle orders: {}, vertices: {:?}, line: {:?}, triangle: {:?}",
            terrain_chunk_coord,
            geometry_data.vertices.len(),
            geometry_data.geometry_data_orders.len(),
            vertex_order_counter,
            line_order_counter,
            triangle_order_counter,
            line_indices.len(),
            line_indices,
            triangle_indices,
        );
    }
}
