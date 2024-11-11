pub mod file_layer;

use std::ops::Not;

use bevy::{log::BoxedLayer, prelude::*};
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Resource)]
pub struct LogLayerGuardRes {
    pub worker_guard_vec: Vec<WorkerGuard>,
}

#[derive(Resource)]
pub struct LogLayerRes {
    pub layer_fns: Vec<BoxedLayerFn>,
}

pub type BoxedLayerFn = Box<dyn Fn(&mut App) -> Option<BoxedLayer> + Send + Sync>;

#[derive(Default)]
pub struct LogLayersPlugin;

impl Plugin for LogLayersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogLayerGuardRes {
            worker_guard_vec: Vec::new(),
        });
        app.insert_resource(LogLayerRes {
            layer_fns: Vec::new(),
        });
    }
}

impl LogLayersPlugin {
    pub fn add_layer(app: &mut App, layer_fn: BoxedLayerFn) {
        let mut log_layer_res = app.world_mut().resource_mut::<LogLayerRes>();
        log_layer_res.layer_fns.push(layer_fn);
    }

    pub fn get_layer(app: &mut App) -> Option<BoxedLayer> {
        LogLayersPlugin::get_layers(app)
    }

    pub fn get_layers(app: &mut App) -> Option<BoxedLayer> {
        let mut layers = vec![];
        let log_layer_res = app
            .world_mut()
            .remove_resource::<LogLayerRes>()
            .expect("LogLayerRes is None, maybe you invoked this function multiple times");
        for layer_fn in log_layer_res.layer_fns.iter() {
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
