use bevy::prelude::*;
use lightyear::prelude::{
    client::{ComponentSyncMode, Predicted},
    AppComponentExt, ChannelDirection,
};
use monster::Monster;
use npc::Npc;
use player::{BornLocation, Player, PlayerId};

use crate::ai::{AiClientPlugin, AiServerPlugin};

pub mod attr_set;
pub mod base;
pub mod monster;
pub mod npc;
pub mod player;

pub type UnitQueryFilter = Or<(With<Player>, With<Monster>, With<Npc>)>;
pub type PredictedUnitQueryFilter = (With<Predicted>, UnitQueryFilter);

#[derive(Default, Debug)]
pub struct UnitClientPlugin;

impl Plugin for UnitClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AiClientPlugin);
    }
}

#[derive(Default, Debug)]
pub struct UnitServerPlugin;

impl Plugin for UnitServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AiServerPlugin);
    }
}

pub(crate) struct UnitProtocolPlugin;

impl Plugin for UnitProtocolPlugin {
    fn build(&self, app: &mut App) {
        // components
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<Player>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<Npc>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<Monster>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<BornLocation>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
    }
}
