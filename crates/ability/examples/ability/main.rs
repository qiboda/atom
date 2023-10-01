mod attribute;
mod base_attack;

use ability::{
    ability::{Ability, AbilityGraph, AbilityPlugin, AbilityState},
    bundle::{AbilityBundle, AbilitySubsystemBundle},
    graph::{
        base::{
            entry::EffectNodeEntryPlugin, log::EffectNodeLogPlugin,
            multiple::EffectNodeMultiplePlugin, timer::EffectNodeTimerPlugin,
        },
        bundle::EffectGraphBundle,
        context::{EffectGraphContext, GraphRef},
        event::EffectEvent,
        EffectGraphPlugin, EffectNodeGraphPlugin,
    },
};
use attribute::BaseAttributeSet;
use base_attack::EffectNodeGraphBaseAttack;

use bevy::{
    prelude::{
        info, App, BuildChildren, Commands, Component, DespawnRecursiveExt, Entity, EventWriter,
        Input, KeyCode, Query, Res, ResMut, Startup, Update, With,
    },
    DefaultPlugins,
};

#[derive(Component)]
struct Player;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EffectGraphPlugin::default())
        .add_plugins(EffectNodeLogPlugin::default())
        .add_plugins(EffectNodeTimerPlugin)
        .add_plugins(EffectNodeEntryPlugin::default())
        .add_plugins(EffectNodeMultiplePlugin::default())
        .add_plugins(EffectNodeGraphPlugin::<EffectNodeGraphBaseAttack>::default())
        .add_plugins(AbilityPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, cast_base_skill)
        .add_systems(Update, remove_base_skill)
        // .add_systems(Last, reset_graph_node_state)
        .run();
}

fn startup(mut commands: Commands, mut ability_graph: ResMut<AbilityGraph>) {
    let player_entity = commands.spawn(Player).id();
    // init attr from player data.
    let ability_subsystem_entity = commands
        .spawn(AbilitySubsystemBundle::<BaseAttributeSet>::default())
        .set_parent(player_entity)
        .id();
    // init ability from player data.
    let base_attack_graph_entity = commands
        .spawn(EffectGraphBundle::<EffectNodeGraphBaseAttack>::default())
        .id();

    let ability_entity = commands
        .spawn(AbilityBundle {
            ability: Ability,
            tag_contaier: Default::default(),
            state: AbilityState::Unactived,
        })
        .set_parent(ability_subsystem_entity)
        .id();

    ability_graph
        .map
        .insert(ability_entity, GraphRef::new(base_attack_graph_entity));
}

/// only can cast once, because node has not reset state.
fn cast_base_skill(
    input: Res<Input<KeyCode>>,
    query: Query<&EffectGraphContext>,
    mut ability_query: Query<(Entity, &mut AbilityState)>,
    mut event_writer: EventWriter<EffectEvent>,
    ability_graph: Res<AbilityGraph>,
) {
    if input.just_pressed(KeyCode::Q) {
        info!("just_pressed: cast_base_skill");
        for (entity, state) in ability_query.iter_mut() {
            if *state == AbilityState::Unactived {
                if let Some(graph) = ability_graph.map.get(&entity) {
                    let graph_context = query.get(graph.get_entity()).unwrap();
                    if let Some(entry_node) = graph_context.entry_node {
                        // ability.set_state(AbilityState::Actived);
                        let event = EffectEvent::Start(entry_node);
                        event_writer.send(event);
                    }
                }
            }
        }
    }
}

fn remove_base_skill(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    ability_query: Query<Entity, With<Ability>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        info!("just_pressed: remove_base_skill");
        for ability_entity in ability_query.iter() {
            commands.entity(ability_entity).despawn_recursive()
        }
    }
}
