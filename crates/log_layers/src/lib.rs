pub mod file_layer;

use std::ops::Not;

use bevy::{log::BoxedLayer, prelude::*};
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Resource)]
pub struct LogLayerRes {
    pub worker_guard_vec: Vec<WorkerGuard>,
}

pub type BoxedLayerFn = fn(app: &mut App) -> Option<BoxedLayer>;

#[derive(Default, Clone)]
pub struct LogLayersPlugin {
    pub layer_fns: Vec<BoxedLayerFn>,
}

impl Plugin for LogLayersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogLayerRes {
            worker_guard_vec: Vec::new(),
        });
    }
}

impl LogLayersPlugin {
    pub fn add_layer(mut self, layer_fn: BoxedLayerFn) -> Self {
        self.layer_fns.push(layer_fn);
        self
    }

    pub fn get_layer(app: &mut App) -> Option<BoxedLayer> {
        let plugins = app.get_added_plugins::<LogLayersPlugin>();
        assert!(plugins.len() == 1, "LogLayersPlugin added only 1");
        plugins[0].clone().get_layers(app)
    }

    pub fn get_layers(&self, app: &mut App) -> Option<BoxedLayer> {
        let mut layers = vec![];
        for layer_fn in self.layer_fns.iter() {
            if let Some(layer) = layer_fn(app) {
                layers.push(layer);
            }
        }

        if layers.is_empty().not() {
            return Some(Box::new(layers));
        }
        None
    }
}
