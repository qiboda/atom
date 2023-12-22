use bevy::math::Vec3;
use enum_kinds::EnumKind;
use serde::{Deserialize, Deserializer, Serialize};
use std::num::NonZeroU64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VertexData {
    pub index: usize,
    #[serde(deserialize_with = "de_vec3", serialize_with = "se_vec3")]
    pub location: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeData {
    pub start_index: usize,
    pub end_index: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TriangleData {
    pub vertex_index_0: usize,
    pub vertex_index_1: usize,
    pub vertex_index_2: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumKind)]
#[enum_kind(OrderTypeKind)]
pub enum OrderType {
    Vertex(VertexData),
    Edge(EdgeData),
    Triangle(TriangleData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fields {
    pub order_type: OrderType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub order_id: NonZeroU64,
    pub level: String,
    pub fields: Fields,
    pub target: String,
    pub spans: Vec<Span>,
    pub thread_id: NonZeroU64,
}

#[derive(Debug, Clone)]
pub struct ThreadId(pub NonZeroU64);

impl Serialize for ThreadId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("ThreadId({})", self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ThreadId {
    fn deserialize<D>(deserializer: D) -> Result<ThreadId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches("ThreadId(").trim_end_matches(')');
        let id = s.parse::<u64>().unwrap();
        Ok(ThreadId(NonZeroU64::new(id).unwrap()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Span {
    pub name: String,
    pub terrain_chunk_coord: Option<[i64; 3]>,
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

fn se_vec3<S>(v: &Vec3, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    format!("Vec3({}, {}, {})", v.x, v.y, v.z).serialize(s)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use crate::order::Order;

    #[test]
    fn test_order_deserialize() {
        let json = r#"{"fields":{"order_type":{"Vertex":{"index":3,"location":"Vec3(0.1875, 0, 0)"}}},"level":"Level(Trace)","name":"event crates\\terrain\\src\\terrain\\trace\\mod.rs:68","order_id":15,"spans":[{"fields":{},"level":"Level(Trace)","name":"dual_contour","target":"terrain_trace"}],"target":"terrain_trace","thread_id":6}"#;

        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.thread_id, NonZeroU64::new(6).unwrap());
    }
}
