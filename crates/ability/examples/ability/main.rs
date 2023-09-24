mod attribute;
mod base_attack;

use ability::{
    ability::AbilityBase,
    bundle::{AbilityBundle, AbilitySubsystemBundle},
    nodes::{
        base::{
            entry::EffectNodeEntryPlugin, msg::EffectNodeMsgPlugin,
            multiple::EffectNodeMultiplePlugin, timer::EffectNodeTimerPlugin,
        },
        event::EffectEvent,
        graph::{EffectGraphBundle, EffectGraphContext},
        EffectGraphPlugin,
    },
};
use attribute::BaseAttributeSet;
use base_attack::{EffectNodeGraphBaseAttack, EffectNodeGraphBaseAttackPlugin};

use bevy::{
    prelude::{
        App, BuildChildren, Commands, Component, EventWriter, Input, KeyCode, Query, Res, Startup,
        Update,
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
        .add_plugins(EffectNodeGraphBaseAttackPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, cast_base_skill)
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
    ability_query: Query<&AbilityBase>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    if input.just_pressed(KeyCode::Q) {
        for ability in ability_query.iter() {
            if let Some(graph) = ability.get_graph() {
                let graph_context = query.get(graph).unwrap();
                if let Some(entry_node) = graph_context.entry_node {
                    let event = EffectEvent::Start(entry_node);
                    event_writer.send(event);
                }
            }
        }
    }
}
