use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use client::{ComponentSyncMode, LerpFn};
use datatables::TableProtocolPlugin;
use lightyear::{
    prelude::*,
    utils::{
        avian3d::{position, rotation},
        bevy::TransformLinearInterpolation,
    },
};

use crate::{input::setting::PlayerAction, unit::UnitProtocolPlugin};

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
        app.add_plugins(UnitProtocolPlugin);

        // messages
        app.register_message::<TestMessage>(ChannelDirection::Bidirectional);

        // inputs
        app.add_plugins(LeafwingInputPlugin::<PlayerAction>::default());

        // components
        app.register_component::<Transform>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_interpolation_fn(TransformLinearInterpolation::lerp);

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
