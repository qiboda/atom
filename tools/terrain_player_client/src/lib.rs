use std::num::NonZeroU64;

use bevy::math::Vec3;
use enum_kinds::EnumKind;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VertexData {
    pub index: usize,
    #[serde(deserialize_with = "de_vec3", serialize_with = "se_vec3")]
    pub location: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeData {
    #[serde(deserialize_with = "de_vec3", serialize_with = "se_vec3")]
    pub start_location: Vec3,
    #[serde(deserialize_with = "de_vec3", serialize_with = "se_vec3")]
    pub end_location: Vec3,
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
pub struct Order {
    pub timestamp: String,
    pub level: String,
    // #[serde(flatten)]
    pub order_type: OrderType,
    pub target: String,
    pub span: Span,
    pub spans: Vec<Span>,
    #[serde(rename = "threadId")]
    pub thread_id: ThreadId,
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
    use super::*;

    #[test]
    fn test_thread_id_serialize() {
        let thread_id = ThreadId(NonZeroU64::new(1).unwrap());
        let json = serde_json::to_string(&thread_id).unwrap();
        assert_eq!(json, r#""ThreadId(1)""#);
    }

    #[test]
    fn test_thread_id_deserialize() {
        let json = r#""ThreadId(1)""#;
        let thread_id: ThreadId = serde_json::from_str(json).unwrap();
        assert_eq!(thread_id.0.get(), 1);
    }

    #[test]
    fn test_order_serialize() {
        let order = Order {
            timestamp: "2021-08-31T09:00:00.000+08:00".to_string(),
            level: "INFO".to_string(),
            order_type: OrderType::Vertex(VertexData {
                index: 0,
                location: Vec3::new(0.0, 0.0, 0.0),
            }),
            target: "terrain::mesh::meshing".to_string(),
            span: Span {
                name: "meshing".to_string(),
            },
            spans: vec![Span {
                name: "meshing".to_string(),
            }],
            thread_id: ThreadId(NonZeroU64::new(1).unwrap()),
        };
        let json = serde_json::to_string(&order).unwrap();
        assert_eq!(
            json,
            r#"{"timestamp":"2021-08-31T09:00:00.000+08:00","level":"INFO","order_type":{"Vertex":{"index":0,"location":"Vec3(0, 0, 0)"}},"target":"terrain::mesh::meshing","span":{"name":"meshing"},"spans":[{"name":"meshing"}],"threadId":"ThreadId(1)"}"#
        );
    }
}
