use bevy::prelude::*;
use serde::{Deserialize, Deserializer};

#[derive(Resource, Default, Debug)]
pub struct PlayerOrders {
    orders: Vec<Order>,
}

impl PlayerOrders {
    pub fn push_order(&mut self, order: Order) {
        self.orders.push(order);
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub index: usize,
    #[serde(deserialize_with = "de_vec3")]
    pub location: Vec3,
    pub target: String,
    pub span: Span,
    pub spans: Vec<Span>,
    pub thread_id: u64,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Span {
    pub name: String,
}

fn de_vec3<'de, D>(de: D) -> Result<Vec3, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    let s = s.trim_start_matches("Vec3(").trim_end_matches(')');
    let mut iter = s.split(", ");
    let x = iter.next().unwrap().parse::<f32>().unwrap();
    let y = iter.next().unwrap().parse::<f32>().unwrap();
    let z = iter.next().unwrap().parse::<f32>().unwrap();
    Ok(Vec3::new(x, y, z))
}
