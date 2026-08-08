//! 属性集：聚合多个 [`Attribute`] 的组件 trait。

use bevy::prelude::Component;

use super::{Attribute, modifier::AttributeModifier};

/// 属性集 trait：按枚举成员索引属性并提供最终值查询与修饰应用。
pub trait AttributeSet: Component {
    /// 属性枚举类型（标识属性集包含哪些属性）。
    type AttributeSetEnum: Copy;

    /// 查询指定属性的最终值。
    fn get_attr_final_value(&self, attribute_set_enum: Self::AttributeSetEnum) -> Option<f32>;

    /// 获取指定属性的不可变引用。
    fn get_attr(&self, attribute_set_enum: Self::AttributeSetEnum) -> &dyn Attribute;

    /// 获取指定属性的可变引用。
    fn get_attr_mut(&mut self, attribute_set_enum: Self::AttributeSetEnum) -> &mut dyn Attribute;

    /// 应用一个修饰符到本属性集。
    fn apply_modify(&mut self, modifier: impl AttributeModifier<AttributeSetType = Self>) {
        modifier.receive_attribute_set(self);
    }
}
