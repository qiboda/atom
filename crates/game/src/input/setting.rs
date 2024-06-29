use bevy::{asset::Asset, prelude::Resource, reflect::Reflect};
use serde::{Deserialize, Serialize};
use settings::Setting;

#[derive(Resource, Serialize, Reflect, Deserialize, Debug, Default, Asset, Clone)]
pub struct InputSetting {}

impl Setting for InputSetting {}
