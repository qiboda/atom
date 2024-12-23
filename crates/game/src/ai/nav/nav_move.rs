use avian3d::prelude::{Collider, ColliderConstructor};
use bevy::prelude::*;
use bevy_landmass::{
    debug::{EnableLandmassDebug, Landmass3dDebugPlugin},
    prelude::*,
};
use bevy_tnua::prelude::{TnuaBuiltinWalk, TnuaController};
use oxidized_navigation::{NavMeshSettings, OxidizedNavigationPlugin};

use super::landmass_navmesh::{LandmassOxidizedNavigationPlugin, OxidizedArchipelago};

pub const COMPUTED_COLLIDER: ColliderConstructor = ColliderConstructor::TrimeshFromMesh;

#[derive(Debug, Default)]
pub struct NavMoveClientPlugin;

impl Plugin for NavMoveClientPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                NavMoveSystemSet::UpdateVelocity,
                NavMoveSystemSet::MoveAgent,
            )
                .chain(),
        )
        .add_plugins(Landmass3dPlugin::default())
        .add_plugins(Landmass3dDebugPlugin::default())
        .add_plugins(LandmassOxidizedNavigationPlugin)
        .insert_resource(EnableLandmassDebug(true))
        .add_plugins(OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings {
            tile_width: 40,
            ..NavMeshSettings::from_agent_and_bounds(0.5, 2.0, 10000.0, -256.0)
        }))
        // .insert_resource(DrawNavMesh(true))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_agent_by_velocity).in_set(NavMoveSystemSet::MoveAgent),
        );
    }
}

#[derive(Debug, Default)]
pub struct NavMoveServerPlugin;

impl Plugin for NavMoveServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_agent_velocity).in_set(NavMoveSystemSet::UpdateVelocity),
        );
    }
}

#[derive(SystemSet, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum NavMoveSystemSet {
    #[default]
    UpdateVelocity,
    MoveAgent,
}

fn setup(mut commands: Commands) {
    let archipelago_entity = commands
        .spawn((
            Archipelago3d::new(AgentOptions::default_for_agent_radius(1.5)),
            OxidizedArchipelago,
        ))
        .id();

    commands.insert_resource(AgentArchipelagoRef { archipelago_entity });

    // COMPUTED_COLLIDER,
    // NavMeshAffector,
}

#[derive(Resource, Debug)]
pub struct AgentArchipelagoRef {
    pub archipelago_entity: Entity,
}

/// Use the desired velocity as the agent's velocity.
fn update_agent_velocity(mut agent_query: Query<(&mut Velocity3d, &AgentDesiredVelocity3d)>) {
    for (mut velocity, desired_velocity) in agent_query.iter_mut() {
        velocity.velocity = desired_velocity.velocity();
    }
}

/// Apply the agent's velocity to its position.
fn move_agent_by_velocity(
    // time: Res<Time>,
    mut agent_query: Query<(&mut TnuaController, &GlobalTransform, &Velocity3d)>,
) {
    for (mut controller, global_transform, velocity) in agent_query.iter_mut() {
        let local_velocity = global_transform
            .affine()
            .inverse()
            .transform_vector3(velocity.velocity);

        info!(
            "local_velocity: {:?}, forward: {:?}",
            local_velocity,
            global_transform.forward().as_vec3()
        );

        controller.basis(TnuaBuiltinWalk {
            desired_velocity: local_velocity,
            desired_forward: Some(global_transform.forward()),
            float_height: 0.0,
            ..Default::default()
        });
    }
}

/// System for toggling the `EnableLandmassDebug` resource.
fn toggle_debug(mut debug: ResMut<EnableLandmassDebug>) {
    **debug = !**debug;
}
