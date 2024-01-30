pub mod load;
pub mod persist;
pub mod plugin;
pub mod setting_path;
pub mod toml_diff;

use bevy::prelude::*;

use serde::{Deserialize, Serialize};

/// settings limits:
///   1. all fields must be Optional
pub trait Settings:
    Resource + Clone + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
{
}

impl<T> Settings for T where
    T: Resource + Clone + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
{
}
