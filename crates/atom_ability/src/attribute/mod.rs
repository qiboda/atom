//! 属性（Attribute）模块：分层属性值与修饰符（Modifier）机制。

use bevy::reflect::Reflect;

pub mod attribute_set;
pub mod implement;
pub mod modifier;

/// 属性层级标识：属性值按层存储，最终值由各层叠加计算。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AttributeLayer(pub &'static str);

// 属性支持范围限定，叠加计算。
// 属性间支持联系，以及属性间范围限定，以及叠加计算。
//
// Note: 暂时不支持任意类型的属性。
/// 分层属性 trait：按层存取属性值并计算最终值。
pub trait Attribute: Reflect {
    /// 读取指定层的属性值。
    fn get_value(&self, layer: AttributeLayer) -> Option<f32>;

    /// 设置指定层的属性值（覆盖）。
    fn set_value(&mut self, layer: AttributeLayer, value: f32);
    /// 在指定层上累加属性值。
    fn add_value(&mut self, layer: AttributeLayer, value: f32);

    /// 读取缓存的最终属性值。
    fn get_final_value(&self) -> f32;

    /// 将 `layer_value` 叠加到指定层后计算最终值。
    fn compute_final_value(&self, layer: AttributeLayer, layer_value: f32) -> f32;

    /// 将最终值误差反算回指定层的值误差（用于超限时削减）。
    fn comptue_error_value(&self, layer: AttributeLayer, final_value_error: f32) -> f32;

    /// 设置最终属性值。
    fn set_final_value(&mut self, final_value: f32);
}
