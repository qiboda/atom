use super::attribute_set::AttributeSet;

pub trait AttributeModifier {
    type AttributeSetType: AttributeSet;

    /// .
    fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType);
}
