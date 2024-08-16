use bevy::app::{PluginGroup, PluginGroupBuilder};
use limit::AtomLimitShaderPlugin;
use math::AtomMathShaderPlugin;
use morton::AtomMortonShaderPlugin;
use noise::AtomNoiseShaderPlugin;
use random::AtomRandomShaderPlugin;

mod gpu_test;
pub mod limit;
pub mod math;
pub mod morton;
pub mod noise;
pub mod random;

#[derive(Default)]
pub struct AtomShaderLibPluginGroups;

impl PluginGroup for AtomShaderLibPluginGroups {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<AtomShaderLibPluginGroups>();
        group
            .add(AtomLimitShaderPlugin)
            .add(AtomMathShaderPlugin)
            .add(AtomNoiseShaderPlugin)
            .add(AtomMortonShaderPlugin)
            .add(AtomRandomShaderPlugin)
    }
}
