use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use client::ComponentSyncMode;
use datatables::TableProtocolPlugin;
use lightyear::{
    prelude::*,
    utils::avian3d::{position, rotation},
};

use crate::{
    input::setting::PlayerAction,
    unit::{
        player::{BornLocation, PlayerId},
        Monster, Npc, Player,
    },
};

#[derive(Channel)]
pub struct DefaultChannel;

// 传输这个结构体，用于同步用户输入
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Inputs {
    None,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TestMessage {
    pub msg: String,
}

pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TableProtocolPlugin);

        // messages
        app.register_message::<TestMessage>(ChannelDirection::Bidirectional);

        // inputs
        app.add_plugins(LeafwingInputPlugin::<PlayerAction>::default());

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

        // app.register_component::<Transform>(ChannelDirection::Bidirectional)
        //     .add_prediction(ComponentSyncMode::Full)
        //     .add_interpolation(ComponentSyncMode::Full)
        //     .add_interpolation_fn(TransformLinearInterpolation::lerp);

        app.register_component::<BornLocation>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        // app.register_component::<Visibility>(ChannelDirection::ServerToClient)
        //     .add_prediction(ComponentSyncMode::Full)
        //     .add_interpolation(ComponentSyncMode::Full);

        {
            app.register_component::<Position>(ChannelDirection::ServerToClient)
                .add_prediction(ComponentSyncMode::Full)
                .add_interpolation(ComponentSyncMode::Full)
                .add_interpolation_fn(position::lerp)
                .add_correction_fn(position::lerp);

            app.register_component::<Rotation>(ChannelDirection::ServerToClient)
                .add_prediction(ComponentSyncMode::Full)
                .add_interpolation(ComponentSyncMode::Full)
                .add_interpolation_fn(rotation::lerp)
                .add_correction_fn(rotation::lerp);

            // NOTE: interpolation/correction is only needed for components that are visually displayed!
            // we still need prediction to be able to correctly predict the physics on the client
            app.register_component::<LinearVelocity>(ChannelDirection::ServerToClient)
                .add_prediction(ComponentSyncMode::Full);

            app.register_component::<AngularVelocity>(ChannelDirection::ServerToClient)
                .add_prediction(ComponentSyncMode::Full);
        }

        // channels
        app.add_channel::<DefaultChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}
