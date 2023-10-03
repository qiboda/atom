use bevy::{
    prelude::{Bundle, Commands, Entity},
    reflect::{reflect_trait, Reflect},
};

use crate::{
    attribute::attribute_set::AttributeSet,
    effect::{state::EffectState, timer::EffectTime},
    stateset::StateLayerTagContainer,
    Ability, Effect,
};

/// ability owner entity
///     ability enitty 1
///     ability base
///        &ability graph
///     ability tag container
///     other
///
///     ability enitty 2
///     ability base
///        &ability graph
///     ability tag container
///     other
///
/// attribute set
/// state set

#[derive(Bundle, Default)]
pub struct AbilitySubsystemBundle<T: AttributeSet> {
    pub attribute_set: T,
    pub state_set: StateLayerTagContainer,
}

#[reflect_trait]
pub trait BundleTrait {
    fn spawn_bundle(&self, commands: &mut Commands) -> Entity;
}

#[reflect_trait]
pub trait AbilityBundleTrait: BundleTrait {}

#[reflect_trait]
pub trait EffectBundleTrait: BundleTrait {}

#[derive(Bundle, Reflect, Clone)]
#[reflect(AbilityBundleTrait)]
pub struct AbilityBundle {
    pub ability: Ability,
    pub state: EffectState,
}

impl BundleTrait for AbilityBundle {
    fn spawn_bundle(&self, commands: &mut Commands) -> Entity {
        commands.spawn(self.clone()).id()
    }
}

impl AbilityBundleTrait for AbilityBundle {}

#[derive(Bundle, Reflect, Clone)]
#[reflect(EffectBundleTrait)]
pub struct TimeEffectBundle {
    pub effect: Effect,
    pub state: EffectState,
    pub time: EffectTime,
}

impl BundleTrait for TimeEffectBundle {
    fn spawn_bundle(&self, commands: &mut Commands) -> Entity {
        commands.spawn(self.clone()).id()
    }
}

impl EffectBundleTrait for TimeEffectBundle {}
