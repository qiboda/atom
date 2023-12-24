use bevy::prelude::Resource;
use terrain_player_client::order::Order;

#[derive(Resource, Default, Debug)]
pub struct Orders {
    pub(crate) orders: Vec<Order>,
}

impl Orders {
    pub fn push_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn get_order(&self, index: usize) -> Option<&Order> {
        self.orders.get(index)
    }
}
