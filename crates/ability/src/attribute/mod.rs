use bevy::reflect::Reflect;

pub mod attribute_set;
pub mod modifier;

// 属性支持范围限定，叠加计算。
// 属性间支持联系，以及属性间范围限定，以及叠加计算。
//
// Note: 暂时不支持任意类型的属性。
pub trait Attribute: Reflect {
    fn get_value(&self) -> &f32;

    fn get_value_mut(&mut self) -> &mut f32;
}
