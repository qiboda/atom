//! 主世界 → 渲染世界的 chunk 处理请求同步。
//! 通过 crossbeam channel 传递 chunk 的加载/卸载指令，避免 ExtractResource 的 sub-app 时序问题。

use bevy::{math::Vec3, prelude::*};
use crossbeam::channel::{Receiver, Sender};

/// 主世界 → 渲染世界的 chunk 处理请求
pub enum ChunkProcessRequest {
    /// 加载新 chunk: entity 和世界坐标
    Load {
        /// chunk 实体 ID
        entity: Entity,
        /// chunk 左下角世界坐标
        world_min: Vec3,
    },
    /// 卸载 chunk: 释放 GPU buffer
    Unload {
        /// 要卸载的 chunk 实体 ID
        entity: Entity,
    },
}

/// 主世界端发送器，包装 crossbeam channel `Sender`
#[derive(Resource, Deref)]
pub struct TerrainChunkProcessSender(pub Sender<ChunkProcessRequest>);

/// 渲染世界端接收器，包装 crossbeam channel `Receiver`
#[derive(Resource, Deref)]
pub struct TerrainChunkProcessReceiver(pub Receiver<ChunkProcessRequest>);
