use avian3d::prelude::{LinearVelocity, Physics, Position, RigidBody, Rotation};
use bevy::{
    app::{Plugin, PostUpdate},
    color::palettes::css,
    diagnostic::LogDiagnosticsPlugin,
    prelude::*,
    time::Time,
};
use bevy_screen_diagnostics::{
    Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin,
    ScreenFrameDiagnosticsPlugin,
};
use lightyear::{
    client::prediction::diagnostics::PredictionDiagnosticsPlugin,
    prelude::{
        client::{
            Confirmed, InterpolationSet, PredictionSet, VisualInterpolateStatus,
            VisualInterpolationPlugin,
        },
        ReplicationGroup,
    },
    transport::io::IoDiagnosticsPlugin,
};
use network::shared::FIXED_TIMESTEP_HZ;

use crate::unit::player::Player;

use super::protocol::ProtocolPlugin;

// For prediction, we want everything entity that is predicted to be part of the same replication group
// This will make sure that they will be replicated in the same message and that all the entities in the group
// will always be consistent (= on the same tick)
pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

#[derive(Debug, Default, Clone)]
pub struct GameSharedPlugin;

impl Plugin for GameSharedPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(ProtocolPlugin)
            // physics
            .insert_resource(Time::new_with(Physics::fixed_once_hz(FIXED_TIMESTEP_HZ)))
            // Screen Diagnostics
            .add_plugins(LogDiagnosticsPlugin {
                filter: Some(vec![
                    IoDiagnosticsPlugin::BYTES_IN,
                    IoDiagnosticsPlugin::BYTES_OUT,
                ]),
                ..default()
            })
            .add_systems(Startup, setup_diagnostic)
            .add_plugins(ScreenDiagnosticsPlugin::default())
            .add_plugins(ScreenEntityDiagnosticsPlugin)
            .add_plugins(ScreenFrameDiagnosticsPlugin)
            // Visual Interpolation
            .add_plugins(VisualInterpolationPlugin::<Position>::default())
            .add_plugins(VisualInterpolationPlugin::<Rotation>::default())
            .observe(add_visual_interpolation_components::<Position>)
            .observe(add_visual_interpolation_components::<Rotation>)
            // draw confirmed shadows
            .add_systems(
                PostUpdate,
                draw_confirmed_shadows
                    .after(InterpolationSet::Interpolate)
                    .after(PredictionSet::VisualCorrection),
            );
    }
}

fn draw_confirmed_shadows(
    mut gizmos: Gizmos,
    confirmed_q: Query<(&Position, &LinearVelocity, &Confirmed), With<Player>>,
    predicted_q: Query<&Position, With<Player>>,
) {
    for (position, velocity, confirmed) in confirmed_q.iter() {
        let speed = velocity.length() / 3.7;
        let ghost_col = css::GRAY.with_alpha(speed);
        gizmos.cuboid(
            Transform::from_translation(**position).with_scale(Vec3::new(0.5, 2.0, 0.5)),
            ghost_col,
        );
        if let Some(e) = confirmed.predicted {
            if let Ok(pos) = predicted_q.get(e) {
                gizmos.line(**position, **pos, ghost_col);
            }
        }
    }
}

fn add_visual_interpolation_components<T: Component>(
    trigger: Trigger<OnAdd, T>,
    q: Query<&RigidBody, With<T>>,
    mut commands: Commands,
) {
    let Ok(rigid_body) = q.get(trigger.entity()) else {
        return;
    };
    // No need to interp static bodies
    if matches!(rigid_body, RigidBody::Static) {
        return;
    }
    // triggering change detection necessary for SyncPlugin to work
    commands
        .entity(trigger.entity())
        .insert(VisualInterpolateStatus::<T> {
            trigger_change_detection: true,
            ..default()
        });
}

fn setup_diagnostic(mut onscreen: ResMut<ScreenDiagnostics>) {
    onscreen
        .add("RB".to_string(), PredictionDiagnosticsPlugin::ROLLBACKS)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
    onscreen
        .add(
            "RBt".to_string(),
            PredictionDiagnosticsPlugin::ROLLBACK_TICKS,
        )
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
    onscreen
        .add(
            "RBd".to_string(),
            PredictionDiagnosticsPlugin::ROLLBACK_DEPTH,
        )
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.1}"));
    // screen diagnostics twitches due to layout change when a metric adds or removes
    // a digit, so pad these metrics to 3 digits.
    onscreen
        .add("KB_in".to_string(), IoDiagnosticsPlugin::BYTES_IN)
        .aggregate(Aggregate::Average)
        .format(|v| format!("{v:0>3.0}"));
    onscreen
        .add("KB_out".to_string(), IoDiagnosticsPlugin::BYTES_OUT)
        .aggregate(Aggregate::Average)
        .format(|v| format!("{v:0>3.0}"));
}
