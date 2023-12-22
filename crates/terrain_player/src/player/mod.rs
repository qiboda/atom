pub mod geometry_data;

use bevy::prelude::*;
use terrain_player_client::order::Order;

#[derive(Resource, Default, Debug)]
pub struct PlayerOrders {
    orders: Vec<Order>,
    current_index: usize,
}

impl PlayerOrders {
    pub fn push_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn get_order(&self, index: usize) -> Option<&Order> {
        self.orders.get(index)
    }
}

impl PlayerOrders {
    pub fn next(&mut self) -> Option<Order> {
        self.current_index += 1;
        let order = self.orders.get(self.current_index);
        order.cloned()
    }

    pub fn pre(&mut self) -> Option<Order> {
        if self.current_index <= 0 {
            return None;
        }
        self.current_index -= 1;
        let order = self.orders.get(self.current_index);
        order.cloned()
    }

    pub fn current(&self) -> Option<Order> {
        let order = self.orders.get(self.current_index);
        order.cloned()
    }
}
