use crate::player::PlayerOrders;
use bevy::prelude::*;
use terrain_player_client::OrderType;

#[derive(Debug, Default, Resource)]
pub struct GeometryData {
    pub vertices: Vec<[f32; 3]>,
    pub line_indices: Vec<u32>,
    pub triangle_indices: Vec<u32>,
}

pub fn process_player_order(
    mut geometry_data: ResMut<GeometryData>,
    player_orders: Res<PlayerOrders>,
) {
    for order in &player_orders.orders {
        match &order.fields.order_type {
            OrderType::Vertex(vertex_data) => {
                geometry_data.vertices.push(vertex_data.location.to_array());
            }
            OrderType::Edge(edge_data) => {
                geometry_data
                    .line_indices
                    .push(edge_data.start_index as u32);
                geometry_data.line_indices.push(edge_data.end_index as u32);
            }
            OrderType::Triangle(triangle_data) => {
                geometry_data
                    .triangle_indices
                    .push(triangle_data.vertex_index_0 as u32);
                geometry_data
                    .triangle_indices
                    .push(triangle_data.vertex_index_1 as u32);
                geometry_data
                    .triangle_indices
                    .push(triangle_data.vertex_index_2 as u32);
            }
        }
    }
}
