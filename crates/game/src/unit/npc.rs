use bevy::prelude::*;
use bevy_landmass::{Agent, Agent3dBundle, AgentSettings, ArchipelagoRef3d};
use lightyear::prelude::client::Predicted;
use serde::{Deserialize, Serialize};

use super::base::{ClientUnitBundle, ServerUnitBundle};

#[derive(Debug, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Npc;

pub type PredictedNpcFilter = (With<Predicted>, With<Npc>);

#[derive(Bundle)]
pub struct ClientNpcBundle {
    unit_bundle: ClientUnitBundle,
    agent_bundle: Agent3dBundle,
}

impl ClientNpcBundle {
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

#[derive(Bundle)]
pub struct ServerNpcBundle {
    pub unit_bundle: ServerUnitBundle,
    pub npc: Npc,
}
