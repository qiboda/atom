use std::ops::Deref;

use bevy::prelude::*;

use crate::layertag::LayerTag;

#[derive(Debug, Clone, Reflect)]
pub struct CountLayerTag {
    layertag: LayerTag,
    counter: i32,
}

impl Deref for CountLayerTag {
    type Target = LayerTag;

    fn deref(&self) -> &Self::Target {
        &self.layertag
    }
}

impl CountLayerTag {
    pub(crate) fn new(layertag: LayerTag) -> Self {
        Self {
            layertag,
            counter: 0,
        }
    }

    pub fn layertag(&self) -> &LayerTag {
        &self.layertag
    }
}

impl CountLayerTag {
    pub fn increase_count(&mut self) {
        self.counter += 1;
    }

    pub fn decrease_count(&mut self) {
        self.counter -= 1;
        if self.counter < 0 {
            trace!(
                "decrease counter to {}, don't should lesser than 0",
                self.counter
            );
        }
    }

    pub fn count(&self) -> i32 {
        self.counter
    }

    pub fn reset_count(&mut self) {
        self.counter = 0;
    }
}
