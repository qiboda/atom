use bevy::app::{App, Plugin};
use nav::nav_move::NavMovePlugin;

pub mod brain;
pub mod nav;
pub mod targets;

/// 发现敌人, 感知系统和敌对系统
/// 调整站位(技能释放需要的位置)(导航的信息)
/// 释放技能
/// 血量太低，逃跑。
/// 逃跑时回血，且无视攻击。
///
/// 组队怪物，移动，攻击，逃跑等。
///

#[derive(Debug, Default)]
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(targets::TargetsPlugin)
            .add_plugins(brain::AiBrainPlugin)
            .add_plugins(NavMovePlugin);
    }
}
