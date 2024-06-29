/// 改变父Entity的Transform。
/// 添加死亡高度，避免子弹一直下落，没有死亡。
///
/// 销毁时机
/// 更新位移和朝向？
/// 目标点
/// 是否hit
///
/// 反弹次数(只有方向型的可以反弹)
/// 分段
/// 受重力影响。
///
/// 轨迹
///
/// 位置 -> 不适用重力。
/// 设置位置曲线，和速度曲线。
///
/// 方向 -> 不固定位置，可能受重力影响(有碰撞体)
/// 速度, 加速度, 方向。(不适应速度曲线)，可能很久都无法销毁。
/// 时间不固定，位置可能也不固定。
///
/// 跟踪 -> 朝向变化
/// 旋转(转向)速度曲线(角速度，角加速度)，匀速
pub mod event;
pub mod plugin;
pub mod movement;

use bevy::{
    math::Vec3,
    prelude::{Component, CubicBSpline, Entity},
};

#[derive(Debug, Default, Component)]
pub struct Projectile;

#[derive(Debug, Component)]
pub enum ProjectileHit {
    None,
    Hit(u32),
}

#[derive(Debug, Component)]
pub enum ProjectileDestroyTime {
    None,
    Hit(f32),
    End(f32),
}

#[derive(Debug, Component)]
pub enum ProjectileDestroyEntity {
    This,
    Parent(Entity),
}

#[derive(Debug, Component)]
pub struct TranslationLockFreedom {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

#[derive(Debug, Component)]
pub struct RotationLockFreedom {
    pub pitch: bool,
    pub yaw: bool,
    pub roll: bool,
}

#[derive(Debug, Component)]
pub enum TransformLockFreedom {
    None,
    Lock(TranslationLockFreedom, RotationLockFreedom),
}
