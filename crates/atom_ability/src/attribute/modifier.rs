//! 属性修饰符：对 [`AttributeSet`] 实施一次性属性调整。

use super::attribute_set::AttributeSet;

/// 属性修饰符 trait：将自身应用到目标属性集上。
pub trait AttributeModifier {
    /// 修饰符作用的目标属性集类型。
    type AttributeSetType: AttributeSet;

    /// .
    /// 将修饰效果应用到 `attribute_set`。
    fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType);
}
