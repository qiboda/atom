use bevy::reflect::Reflect;

pub mod attribute_set;
pub mod implement;
pub mod modifier;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AttributeLayer(pub &'static str);

// 属性支持范围限定，叠加计算。
// 属性间支持联系，以及属性间范围限定，以及叠加计算。
//
// Note: 暂时不支持任意类型的属性。
pub trait Attribute: Reflect {
    fn get_value(&self, layer: AttributeLayer) -> Option<f32>;

    fn set_value(&mut self, layer: AttributeLayer, value: f32);
    fn add_value(&mut self, layer: AttributeLayer, value: f32);

    fn get_final_value(&self) -> f32;

    fn compute_final_value(&self, layer: AttributeLayer, layer_value: f32) -> f32;

    fn comptue_error_value(&self, layer: AttributeLayer, final_value_error: f32) -> f32;

    fn set_final_value(&mut self, final_value: f32);
}
