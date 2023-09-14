// a skill
//     effect graph child
//         effect nodes children nodes.
//

use bevy::prelude::*;

use crate::nodes::{
    base::{msg::MsgNodeBundle, timer::TimerNodeBundle},
    graph::{EffectGraph, EffectGraphBuilder},
};

#[derive(Debug, Component)]
pub struct EffectNodeGraphBaseAttack {}

impl EffectGraphBuilder for EffectNodeGraphBaseAttack {
    fn build(&self, commands: &mut Commands) {
        let msg_node = commands.spawn(MsgNodeBundle::new("base attack")).id();

        let mut timer_node = TimerNodeBundle::new(3.0);
        timer_node.timer.end_exec.entities.push(msg_node);
        commands.spawn(timer_node);
    }
}

impl EffectGraph for EffectNodeGraphBaseAttack {}
