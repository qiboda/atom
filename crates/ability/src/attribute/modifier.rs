pub trait AttributeModifier {
    type AttributeSetType;
    fn receive_attribute_set(&self, attribute_set: &mut Self::AttributeSetType);
}
