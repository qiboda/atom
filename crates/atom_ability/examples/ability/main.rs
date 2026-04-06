mod attribute;
mod base_attack;

use atom_ability::{
    AbilitySubsystemPlugin,
    ability::{
        bundle::{AbilityBundle, AbilityOwnerBundle},
        comp::Ability,
        event::{AbilityRemoveEvent, AbilityStartEvent},
    },
    buff::node::buff_entry::EffectNodeBuffEntryPlugin,
    graph::{
        graph_map::EffectGraphBuilderMapExt,
        node::implement::{
            log::EffectNodeLogPlugin, seq::EffectNodeSeqPlugin, timer::EffectNodeTimerPlugin,
        },
    },
};

use attribute::BaseAttributeSet;
use base_attack::EffectNodeGraphBaseAttack;

use atom_datatables::{
    DataTablePlugin,
    effect::{TbAbility, TbAbilityRow},
    tables_system_param::TableReader,
};
use bevy::{DefaultPlugins, input::ButtonInput, log::info, prelude::*};

#[derive(Component, Reflect)]
struct Player;

fn main() {
    dotenv::dotenv().ok();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(DataTablePlugin)
        .add_plugins(AbilitySubsystemPlugin)
        .add_plugins(EffectNodeTimerPlugin)
        .add_plugins(EffectNodeLogPlugin)
        .add_plugins(EffectNodeSeqPlugin)
        .add_plugins(EffectNodeBuffEntryPlugin)
        .register_effect_graph_builder::<EffectNodeGraphBaseAttack>()
        .add_systems(Update, create_ability)
        .add_systems(Update, cast_base_skill)
        .add_systems(Update, remove_base_skill)
        .run();
}

fn create_ability(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    ability_reader: TableReader<TbAbility>,
    query: Query<(), With<Ability>>,
) {
    if input.just_pressed(KeyCode::KeyC) {
        if query.iter().count() > 0 {
            return;
        }

        let Some(row_data) = ability_reader.get_row(&1) else {
            return;
        };

        commands
            .spawn((Player, AbilityOwnerBundle::<BaseAttributeSet>::default()))
            .with_children(|parent| {
                parent.spawn(AbilityBundle {
                    ability_row: TbAbilityRow {
                        key: 1,
                        data: Some(row_data),
                    },
                    ..Default::default()
                });
            });

        info!("create_ability");
    }
}

/// only can cast once, because node has not reset state.
fn cast_base_skill(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    ability_query: Query<Entity, With<Ability>>,
) {
    if input.just_pressed(KeyCode::KeyQ) {
        info!("just_pressed: cast_base_skill");
        for entity in ability_query.iter() {
            commands.trigger(AbilityStartEvent {
                ability_entity: entity,
            });
        }
    }
}

fn remove_base_skill(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    ability_query: Query<Entity, With<Ability>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        info!("just_pressed: remove_base_skill");
        for ability_entity in ability_query.iter() {
            commands.trigger(AbilityRemoveEvent { ability_entity });
        }
    }
}
