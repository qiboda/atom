use bevy::prelude::*;
use lightyear::prelude::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct PlayerId(pub ClientId);

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct BornLocation(pub Vec3);
