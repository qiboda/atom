use bevy::prelude::*;
use terrain_player_client::Order;

#[derive(Resource, Default, Debug)]
pub struct PlayerOrders {
    orders: Vec<Order>,
}

impl PlayerOrders {
    pub fn push_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn get_order(&self, index: usize) -> Option<&Order> {
        self.orders.get(index)
    }
}
