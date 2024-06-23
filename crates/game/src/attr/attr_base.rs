use ability::attribute::{Attribute, AttributeLayer};
use bevy::reflect::Reflect;

pub const BASE_VALUE_LAYER: AttributeLayer = AttributeLayer("base_value");
pub const BASE_PERCENT_LAYER: AttributeLayer = AttributeLayer("base_percent");
pub const ITEM_VALUE_LAYER: AttributeLayer = AttributeLayer("item_value");
pub const ITEM_PERCENT_LAYER: AttributeLayer = AttributeLayer("item_percnet");
pub const BUFF_VALUE_LAYER: AttributeLayer = AttributeLayer("buff_value");
pub const BUFF_PERCENT_LAYER: AttributeLayer = AttributeLayer("buff_percent");
pub const NONE_LAYER: AttributeLayer = AttributeLayer("none");

#[derive(Debug, Default, Reflect)]
pub struct ValueAttribute {
    base_value: f32,
    item_value: f32,
    buff_value: f32,
    cached_final_value: f32,
}

impl ValueAttribute {
    pub fn new(base_value: f32, item_value: f32, buff_value: f32) -> Self {
        let mut s = Self {
            base_value,
            item_value,
            buff_value,
            cached_final_value: 0.0,
        };

        let final_value = s.compute_final_value(NONE_LAYER, 0.0);
        s.set_final_value(final_value);
        s
    }
}

impl Attribute for ValueAttribute {
    fn get_value(&self, layer: AttributeLayer) -> Option<f32> {
        match layer {
            BASE_VALUE_LAYER => Some(self.base_value),
            ITEM_VALUE_LAYER => Some(self.item_value),
            BUFF_VALUE_LAYER => Some(self.buff_value),
            _ => None,
        }
    }

    fn set_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value = value,
            ITEM_VALUE_LAYER => self.item_value = value,
            BUFF_VALUE_LAYER => self.buff_value = value,
            _ => panic!("set_value error: layer not found!"),
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn add_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value += value,
            ITEM_VALUE_LAYER => self.item_value += value,
            BUFF_VALUE_LAYER => self.buff_value += value,
            _ => panic!("set_value error: layer not found!"),
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn get_final_value(&self) -> f32 {
        self.cached_final_value
    }

    fn compute_final_value(&self, layer: AttributeLayer, layer_value: f32) -> f32 {
        match layer {
            BASE_VALUE_LAYER => self.base_value + self.item_value + self.buff_value + layer_value,
            ITEM_VALUE_LAYER => self.base_value + self.item_value + self.buff_value + layer_value,
            BUFF_VALUE_LAYER => self.base_value + self.item_value + self.buff_value + layer_value,
            _ => self.base_value + self.item_value + self.buff_value,
        }
    }

    fn set_final_value(&mut self, final_value: f32) {
        self.cached_final_value = final_value;
    }

    fn comptue_error_value(&self, _layer: AttributeLayer, final_value_error: f32) -> f32 {
        final_value_error
    }
}

// #[derive(Debug, Default, Reflect)]
// pub(crate) struct PercentAttribute {
//     base_percent: f32,
//     item_percent: f32,
//     buff_percent: f32,
// }

// impl Attribute for PercentAttribute {
//     fn get_value(&self, layer: AttributeLayer) -> Option<f32> {
//         match layer {
//             BASE_PERCENT_LAYER => Some(self.base_percent),
//             ITEM_PERCENT_LAYER => Some(self.item_percent),
//             BUFF_PERCENT_LAYER => Some(self.buff_percent),
//             _ => None,
//         }
//     }

//     fn get_value_mut(&mut self, layer: AttributeLayer) -> Option<&mut f32> {
//         match layer {
//             BASE_PERCENT_LAYER => Some(&mut self.base_percent),
//             ITEM_PERCENT_LAYER => Some(&mut self.item_percent),
//             BUFF_PERCENT_LAYER => Some(&mut self.buff_percent),
//             _ => None,
//         }
//     }

//     fn get_final_value(&self) -> f32 {
//         self.base_percent + self.item_percent + self.buff_percent
//     }
// }

#[derive(Debug, Default, Reflect)]
pub(crate) struct ValuePercentAttribute {
    base_value: f32,
    item_value: f32,
    buff_value: f32,
    base_percent: f32,
    item_percent: f32,
    buff_percent: f32,
    cached_final_value: f32,
}

impl Attribute for ValuePercentAttribute {
    fn get_value(&self, layer: AttributeLayer) -> Option<f32> {
        match layer {
            BASE_VALUE_LAYER => Some(self.base_value),
            ITEM_VALUE_LAYER => Some(self.item_value),
            BUFF_VALUE_LAYER => Some(self.buff_value),
            BASE_PERCENT_LAYER => Some(self.base_percent),
            ITEM_PERCENT_LAYER => Some(self.item_percent),
            BUFF_PERCENT_LAYER => Some(self.buff_percent),
            _ => None,
        }
    }

    fn set_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value = value,
            ITEM_VALUE_LAYER => self.item_value = value,
            BUFF_VALUE_LAYER => self.buff_value = value,
            BASE_PERCENT_LAYER => self.base_percent = value,
            ITEM_PERCENT_LAYER => self.item_percent = value,
            BUFF_PERCENT_LAYER => self.buff_percent = value,
            _ => {
                panic!("set_value error: layer not found!")
            }
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn add_value(&mut self, layer: AttributeLayer, value: f32) {
        match layer {
            BASE_VALUE_LAYER => self.base_value += value,
            ITEM_VALUE_LAYER => self.item_value += value,
            BUFF_VALUE_LAYER => self.buff_value += value,
            BASE_PERCENT_LAYER => self.base_percent += value,
            ITEM_PERCENT_LAYER => self.item_percent += value,
            BUFF_PERCENT_LAYER => self.buff_percent += value,
            _ => {
                panic!("add_value error: layer not found!")
            }
        }

        let final_value = self.compute_final_value(NONE_LAYER, 0.0);
        self.set_final_value(final_value);
    }

    fn get_final_value(&self) -> f32 {
        self.cached_final_value
    }

    fn compute_final_value(&self, layer: AttributeLayer, layer_value: f32) -> f32 {
        match layer {
            BASE_VALUE_LAYER => {
                (self.base_value + layer_value) * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            ITEM_VALUE_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + (self.item_value + layer_value) * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            BUFF_VALUE_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + (self.buff_value + layer_value) * (1.0 + self.buff_percent)
            }
            BASE_PERCENT_LAYER => {
                self.base_value * (1.0 + self.base_percent + layer_value)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            ITEM_PERCENT_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent + layer_value)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
            BUFF_PERCENT_LAYER => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent + layer_value)
            }
            _ => {
                self.base_value * (1.0 + self.base_percent)
                    + self.item_value * (1.0 + self.item_percent)
                    + self.buff_value * (1.0 + self.buff_percent)
            }
        }
    }

    fn set_final_value(&mut self, final_value: f32) {
        self.cached_final_value = final_value;
    }

    fn comptue_error_value(&self, layer: AttributeLayer, final_value_error: f32) -> f32 {
        match layer {
            BASE_VALUE_LAYER => final_value_error / (1.0 + self.base_percent),
            ITEM_VALUE_LAYER => final_value_error / (1.0 + self.item_percent),
            BUFF_VALUE_LAYER => final_value_error / (1.0 + self.buff_percent),
            BASE_PERCENT_LAYER => final_value_error / self.base_value - 1.0,
            ITEM_PERCENT_LAYER => final_value_error / self.item_value - 1.0,
            BUFF_PERCENT_LAYER => final_value_error / self.buff_value - 1.0,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_final_value() {
        let attr = ValueAttribute::new(1.0, 2.0, 3.0);
        assert_eq!(attr.get_final_value(), 6.0);
    }

    #[test]
    fn test_get_value() {
        let attr = ValueAttribute::new(1.0, 2.0, 3.0);
        assert_eq!(attr.get_value(BASE_VALUE_LAYER), Some(1.0));
        assert_eq!(attr.get_value(ITEM_VALUE_LAYER), Some(2.0));
        assert_eq!(attr.get_value(BUFF_VALUE_LAYER), Some(3.0));
    }
}
