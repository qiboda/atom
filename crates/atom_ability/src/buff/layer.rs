use bevy::prelude::*;

#[derive(Component, Debug, Default, Reflect, Clone)]
#[reflect(Component)]
pub struct BuffLayer {
    layer: i32,
    max_layer: i32,
}

impl BuffLayer {
    pub fn new(max_layer: i32) -> Self {
        Self {
            layer: 1,
            max_layer,
        }
    }
}

impl BuffLayer {
    pub fn add_layer(&mut self, layer: i32) {
        assert!(layer > 0, "layer must be greater than 0");
        self.layer += layer;
        self.layer = self.layer.min(self.max_layer);
    }

    pub fn remove_layer(&mut self, layer: i32) {
        assert!(layer > 0, "layer must be greater than 0");
        self.layer -= layer;
        self.layer = self.layer.max(0);
    }
}
