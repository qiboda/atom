// mod attribute;
// mod base_attack;

// use ability::{
//     bundle::{AbilityBundle, AbilitySubsystemBundle},
//     effect::{
//         event::EffectStartEvent, graph_map::EffectGraphMap, state::EffectState, EffectPlugin,
//     },
//     graph::{
//         base::{
//             entry::EffectNodeEntryPlugin, log::EffectNodeLogPlugin,
//             multiple::EffectNodeMultiplePlugin, timer::EffectNodeTimerPlugin,
//         },
//         bundle::EffectGraphBundle,
//         context::GraphRef,
//         event::EffectNodeEventPlugin,
//         EffectGraphPlugin, EffectNodeGraphPlugin,
//     },
//     Ability,
// };
// use attribute::BaseAttributeSet;
// use base_attack::EffectNodeGraphBaseAttack;

// use bevy::{
//     input::ButtonInput,
//     log::info,
//     prelude::{
//         App, BuildChildren, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, KeyCode,
//         Query, Res, ResMut, Startup, Update, With,
//     },
//     reflect::Reflect,
//     DefaultPlugins,
// };

// #[derive(Component, Reflect)]
// struct Player;

// fn main() {
//     let mut app = App::new();
//     app.add_plugins(DefaultPlugins)
//         .add_plugins(EffectGraphPlugin::default())
//         .add_plugins(EffectNodeEventPlugin)
//         .add_plugins(EffectNodeLogPlugin::default())
//         .add_plugins(EffectNodeTimerPlugin)
//         .add_plugins(EffectNodeEntryPlugin::default())
//         .add_plugins(EffectNodeMultiplePlugin::default())
//         .add_plugins(EffectNodeGraphPlugin::<EffectNodeGraphBaseAttack>::default())
//         .add_plugins(EffectPlugin)
//         .add_systems(Startup, startup)
//         .add_systems(Update, cast_base_skill)
//         .add_systems(Update, remove_base_skill)
//         .run();
// }

// fn startup(mut commands: Commands, mut ability_graph: ResMut<EffectGraphMap>) {
//     let player_entity = commands.spawn(Player).id();
//     // init attr from player data.
//     let ability_subsystem_entity = commands
//         .spawn(AbilitySubsystemBundle::<BaseAttributeSet>::default())
//         .set_parent(player_entity)
//         .id();
//     // init ability from player data.
//     let base_attack_graph_entity = commands
//         .spawn(EffectGraphBundle::<EffectNodeGraphBaseAttack>::default())
//         .id();

//     let ability_entity = commands
//         .spawn(AbilityBundle {
//             ability: Ability,
//             state: EffectState::Inactive,
//         })
//         .set_parent(ability_subsystem_entity)
//         .id();

//     ability_graph
//         .map
//         .insert(ability_entity, GraphRef::new(base_attack_graph_entity));
// }

// /// only can cast once, because node has not reset state.
// fn cast_base_skill(
//     input: Res<ButtonInput<KeyCode>>,
//     mut ability_query: Query<(Entity, &mut EffectState)>,
//     mut event_writer: EventWriter<EffectStartEvent>,
// ) {
//     if input.just_pressed(KeyCode::KeyQ) {
//         info!("just_pressed: cast_base_skill");
//         for (entity, state) in ability_query.iter_mut() {
//             if *state == EffectState::Inactive {
//                 event_writer.send(EffectStartEvent {
//                     effect: entity,
//                     data: None,
//                 });
//             }
//         }
//     }
// }

// fn remove_base_skill(
//     mut commands: Commands,
//     input: Res<ButtonInput<KeyCode>>,
//     ability_query: Query<Entity, With<Ability>>,
// ) {
//     if input.just_pressed(KeyCode::Escape) {
//         info!("just_pressed: remove_base_skill");
//         for ability_entity in ability_query.iter() {
//             commands.entity(ability_entity).despawn_recursive()
//         }
//     }
// }

fn main() {}
