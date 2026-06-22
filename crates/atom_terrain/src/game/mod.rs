use bevy::prelude::*;

/// 俯视角摄像机系统：TopDownCamera 组件 + 玩家跟随。
pub mod camera;
/// 玩家系统：Player/Name/Health/MoveSpeed 组件 + WASD 移动。
pub mod player;

pub use camera::TopDownCamera;
pub use player::{Health, MoveSpeed, Name, Player};

/// 游戏框架根插件，注册所有游戏组件和系统。
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // 注册组件到类型寄存器（BRP 通过 Reflect 访问）
        app.register_type::<Player>();
        app.register_type::<Name>();
        app.register_type::<Health>();
        app.register_type::<MoveSpeed>();
        app.register_type::<TopDownCamera>();

        // 添加系统（player_movement 与 FreeCamera 的 WASD 冲突，暂时移除）
        app.add_systems(
            Update,
            (camera::top_down_camera_follow,),
        );
    }
}
