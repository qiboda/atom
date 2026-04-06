use bevy::{ecs::system::EntityCommands, prelude::Commands, reflect::reflect_trait};

#[reflect_trait]
pub trait BundleTrait {
    fn spawn_bundle<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a>;
}

#[reflect_trait]
pub trait AbilityBundleTrait: BundleTrait {}

#[reflect_trait]
pub trait BuffBundleTrait: BundleTrait {}
