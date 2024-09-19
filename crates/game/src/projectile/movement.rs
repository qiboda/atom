use bevy::prelude::*;
use smallvec::SmallVec;

/// translation or rotation direction
#[derive(Debug, Component)]
pub enum DirectionVariant {
    Constant(Vec3),
    Curve(CubicBSpline<Vec3>),
}

/// translation or rotation speed
#[derive(Debug, Component)]
pub enum SpeedVariant {
    // speed
    Constant(f32),
    // speed and accel
    Derivative(f32, f32),
}

#[derive(Debug, Component)]
pub struct ProjectileMovement {
    pub movements: SmallVec<[ProjectileMovementVariant; 1]>,
}

#[derive(Debug, Component)]
pub enum ProjectileMovementVariant {
    Direction(DirectionMovement),
    Location(LocationMovement),
    TraceTarget(TraceTargetMovement),
}

/// 方向 -> 不固定位置，可能受重力影响(有碰撞体)
/// 速度, 加速度, 方向。(不适应速度曲线)，可能很久都无法销毁。
/// 时间不固定，位置可能也不固定。
#[derive(Debug)]
pub struct DirectionMovement {
    pub speed: SpeedVariant,
    pub direction: Vec3,
}

/// 位置 -> 不适用重力。
/// 设置位置曲线，和速度曲线。
#[derive(Debug)]
pub struct LocationMovement {
    pub location: Vec3,
    pub trajectory: CubicBSpline<Vec3>,
    pub speed: SpeedVariant,
}

/// 跟踪 -> 朝向变化
/// 旋转(转向)速度曲线(角速度，角加速度)，匀速
#[derive(Debug)]
pub struct TraceTargetMovement {
    pub entity: Entity,
    pub rot_speed: SpeedVariant,
    pub translation_speed: SpeedVariant,
}
