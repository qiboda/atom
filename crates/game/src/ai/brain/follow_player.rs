use core::f32;

use bevy::prelude::*;
use bevy::utils::tracing::{debug, trace};
use bevy_landmass::{AgentState, AgentTarget3d};
use big_brain::prelude::*;

use crate::unit::player::PredictedPlayerFilter;
use crate::unit::PredictedUnitQueryFilter;

/// An action where the actor moves to the closest water source
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct MoveToPlayer;

fn move_to_player_action_system(
    mut commands: Commands,
    unit_query: Query<(&GlobalTransform, &AgentState), PredictedUnitQueryFilter>,
    player_query: Query<(Entity, &GlobalTransform), PredictedPlayerFilter>,
    // A query on all current MoveToWaterSource actions.
    mut action_query: Query<(&Actor, &mut ActionState, &MoveToPlayer, &ActionSpan)>,
) {
    // Loop through all actions, just like you'd loop over all entities in any other query.
    for (actor, mut action_state, _move_to_player, span) in &mut action_query {
        let _guard = span.span().enter();

        // Different behavior depending on action state.
        match *action_state {
            // Action was just requested; it hasn't been seen before.
            ActionState::Requested => {
                debug!("Let's go find some water!");

                // Look up the actor's position.
                let (actor_transform, _agent_state) = unit_query
                    .get(actor.0)
                    .expect("actor has no GlobalTransform or not exist");

                trace!("Actor position: {:?}", actor_transform.translation());

                // Look up the water source closest to them.
                let closest_player_entity = find_closest_player(&player_query, actor_transform);

                match closest_player_entity {
                    Some(entity) => {
                        commands
                            .entity(actor.0)
                            .insert(AgentTarget3d::Entity(entity));
                        *action_state = ActionState::Executing;
                    }
                    None => {
                        *action_state = ActionState::Failure;
                    }
                }
            }
            ActionState::Executing => {
                let (_actor_transform, agent_state) = unit_query
                    .get(actor.0)
                    .expect("actor has no GlobalTransform or not exist");

                info!("agent state: {:?}", agent_state);
                match agent_state {
                    AgentState::Idle => {
                        *action_state = ActionState::Failure;
                    }
                    AgentState::ReachedTarget => {
                        *action_state = ActionState::Success;
                    }
                    AgentState::Moving => {
                        *action_state = ActionState::Executing;
                    }
                    AgentState::AgentNotOnNavMesh => {
                        *action_state = ActionState::Failure;
                    }
                    AgentState::TargetNotOnNavMesh => {
                        *action_state = ActionState::Failure;
                    }
                    AgentState::NoPath => {
                        *action_state = ActionState::Failure;
                    }
                }
            }
            ActionState::Cancelled => {
                // Always treat cancellations, or we might keep doing this forever!
                // You don't need to terminate immediately, by the way, this is only a flag that
                // the cancellation has been requested. If the actor is balancing on a tightrope,
                // for instance, you may let them walk off before ending the action.
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// A utility function that finds the closest water source to the actor.
fn find_closest_player(
    player_query: &Query<(Entity, &GlobalTransform), PredictedPlayerFilter>,
    actor_position: &GlobalTransform,
) -> Option<Entity> {
    if let Some((entity, _)) = player_query.iter().min_by(|(_, a), (_, b)| {
        let da = (a.translation() - actor_position.translation()).length_squared();
        let db = (b.translation() - actor_position.translation()).length_squared();
        da.partial_cmp(&db).unwrap()
    }) {
        Some(entity)
    } else {
        None
    }
}

// Scorers are the same as in the thirst example.
#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct TooFar;

pub fn thirsty_scorer_system(
    unit_query: Query<&GlobalTransform, PredictedUnitQueryFilter>,
    player_query: Query<&GlobalTransform, PredictedPlayerFilter>,
    mut query: Query<(&Actor, &mut Score), With<TooFar>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(unit_transform) = unit_query.get(*actor) {
            let mut min_distance = f32::MAX;
            for player in player_query.iter() {
                let distance = unit_transform
                    .translation()
                    .distance_squared(player.translation());
                min_distance = min_distance.min(distance);
            }
            score.set((min_distance / 10.0).min(1.0));
        }
    }
}

pub fn build_ai_entity(cmd: &mut Commands, ai_entity: Entity) {
    cmd.entity(ai_entity).insert(
        build_thinker(), // add some components of thinker needed to the entity
    );
}

pub fn build_thinker() -> ThinkerBuilder {
    // We use the Steps struct to essentially build a "MoveAndDrink" action by composing
    // the MoveToWaterSource and Drink actions.
    //
    // If either of the steps fails, the whole action fails. That is: if the actor somehow fails
    // to move to the water source (which is not possible in our case) they will not attempt to
    // drink either. Getting them un-stuck from that situation is then up to other possible actions.
    //
    // We build up a list of steps that make it so that the actor will...
    // let move_and_drink = Steps::build()
    //     .label("MoveAndDrink")
    //     // ...move to the water source...
    //     .step(MoveToPlayer);

    // Build the thinker
    Thinker::build()
        .label("FollowPlayerThinker")
        // We don't do anything unless we're thirsty enough.
        .picker(Highest)
        .when(TooFar, MoveToPlayer)
}

#[derive(Debug, Default)]
pub struct FollowPlayerThinkerPlugin;

impl Plugin for FollowPlayerThinkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, thirsty_scorer_system.in_set(BigBrainSet::Scorers))
            .add_systems(
                PreUpdate,
                (move_to_player_action_system).in_set(BigBrainSet::Actions),
            );
    }
}
