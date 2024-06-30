use bevy::{asset::Asset, prelude::Resource, reflect::Reflect};
use serde::{Deserialize, Serialize};
use settings_derive::Setting;

#[derive(Resource, Serialize, Reflect, Deserialize, Debug, Default, Asset, Clone, Setting)]
pub struct InputSetting {}
