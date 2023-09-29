mod attribute;
mod base_attack;

use ability::{
    ability::{reset_graph_node_state, AbilityBase, AbilityState},
    bundle::{AbilityBundle, AbilitySubsystemBundle},
    graph::{
        base::{
            entry::EffectNodeEntryPlugin, log::EffectNodeMsgPlugin,
            multiple::EffectNodeMultiplePlugin, timer::EffectNodeTimerPlugin,
        },
        bundle::EffectGraphBundle,
        event::EffectEvent,
        context::EffectGraphContext,
        EffectGraphPlugin, EffectNodeGraphPlugin,
    },
};
use attribute::BaseAttributeSet;
use base_attack::EffectNodeGraphBaseAttack;

use bevy::{
    prelude::{
        App, BuildChildren, Commands, Component, EventWriter, Input, KeyCode, Last, Query, Res,
        Startup, Update,
    },
    DefaultPlugins,
};

#[derive(Component)]
struct Player;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EffectGraphPlugin::default())
        .add_plugins(EffectNodeMsgPlugin::default())
        .add_plugins(EffectNodeTimerPlugin)
        .add_plugins(EffectNodeEntryPlugin::default())
        .add_plugins(EffectNodeMultiplePlugin::default())
        .add_plugins(EffectNodeGraphPlugin::<EffectNodeGraphBaseAttack>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, cast_base_skill)
        .add_systems(Last, reset_graph_node_state)
        .run();
}

fn startup(mut commands: Commands) {
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

    let _ability_entity = commands
        .spawn(AbilityBundle {
            ability: AbilityBase::new(base_attack_graph_entity),
            tag_contaier: Default::default(),
        })
        .set_parent(ability_subsystem_entity)
        .id();
}

/// only can cast once, because node has not reset state.
fn cast_base_skill(
    input: Res<Input<KeyCode>>,
    query: Query<&EffectGraphContext>,
    mut ability_query: Query<&mut AbilityBase>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    if input.just_pressed(KeyCode::Q) {
        for mut ability in ability_query.iter_mut() {
            if ability.get_state() == AbilityState::Unactived {
                if let Some(graph) = ability.get_graph() {
                    let graph_context = query.get(graph).unwrap();
                    if let Some(entry_node) = graph_context.entry_node {
                        ability.set_state(AbilityState::Actived);
                        let event = EffectEvent::Start(entry_node);
                        event_writer.send(event);
                    }
                }
            }
        }
    }
}
