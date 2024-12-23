use bevy::prelude::*;
use bevy_landmass::{Agent, Agent3dBundle, AgentSettings, ArchipelagoRef3d};
use lightyear::prelude::client::Predicted;
use serde::{Deserialize, Serialize};

use super::base::{ClientUnitBundle, ServerUnitBundle};

#[derive(Debug, Default, Component, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Monster;

pub type PredictedMonsterFilter = (With<Predicted>, With<Monster>);

#[derive(Bundle)]
pub struct ClientMonsterBundle {
    pub unit_bundle: ClientUnitBundle,
    pub agent_bundle: Agent3dBundle,
}

impl ClientMonsterBundle {
    fn new(
        radius: f32,
        height: f32,
        desired_speed: f32,
        max_speed: f32,
        archipelago_ref: ArchipelagoRef3d,
    ) -> Self {
        Self {
            unit_bundle: ClientUnitBundle::new(radius, height),
            agent_bundle: Agent3dBundle {
                agent: Agent::default(),
                archipelago_ref,
                settings: AgentSettings {
                    radius,
                    desired_speed,
                    max_speed,
                },
            },
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct ServerMonsterBundle {
    pub unit_bundle: ServerUnitBundle,
    pub monster: Monster,
}
