//! Per-chunk 管理：ChunkId、ChunkState、ChunkManager。
//! 32³ voxel per chunk，+1 ghost border → grid_size = 32。
//! 水平 80m 半径，垂直 -64~32。

use bevy::{
    prelude::*,
    render::extract_resource::ExtractResource,
};
use std::collections::HashMap;

/// Chunk 在网格中的坐标（以 chunk 为单位，不是世界坐标）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkId {
    /// X 轴 chunk 索引
    pub x: i32,
    /// Y 轴 chunk 索引
    pub y: i32,
    /// Z 轴 chunk 索引
    pub z: i32,
}

impl ChunkId {
    /// 创建 ChunkId
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Chunk 原点世界坐标（ghost border 不计入）
    pub fn world_min(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * 15.0,
            self.y as f32 * 15.0,
            self.z as f32 * 15.0,
        )
    }

    /// Chunk 间距（正负 1 在相邻 chunk）
    pub fn distance(&self, other: &ChunkId) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt() * 15.0
    }

    /// 偏移此 ChunkId (dx, dy, dz) 个 chunk 单位
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }
}

/// Chunk GPU compute 生命周期
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChunkState {
    /// 需要创建（尚未分配 slot）
    Pending,
    /// 正在 compute（pass 0~5 尚未完成）
    Computing,
    /// 完成 compute，有有效的 GPU 数据（可渲染）
    Ready,
    /// 标记卸载（即将释放 slot）
    Unloading,
}

/// 加载/卸载请求（主世界 → 渲染世界通信）
#[derive(Resource, ExtractResource, Clone, Default)]
pub struct ChunkLoadRequest {
    /// 主世界计算出的"应当加载"的 chunk 集合
    pub wanted: std::collections::HashSet<ChunkId>,
}

/// 管理多 chunk 生命周期（主世界 Resource，Extract 到渲染世界）
#[derive(Resource, ExtractResource, Clone)]
pub struct ChunkManager {
    /// 所有活跃 chunk：ChunkId → slot index
    pub active: HashMap<ChunkId, usize>,
    /// Voxel 边长
    pub voxel_size: f32,
    /// 水平加载半径（米）
    pub radius: f32,
    /// 地形最低高度
    pub height_min: f32,
    /// 地形最高高度
    pub height_max: f32,
    /// 上次观察者位置
    pub last_observer: Vec3,
    /// 需要的 chunk 集合（用于对比判断哪些需要卸载）
    pub wanted: std::collections::HashSet<ChunkId>,
}

impl ChunkManager {
    /// 创建 chunk 管理器
    pub fn new(radius: f32, height_min: f32, height_max: f32) -> Self {
        Self {
            active: HashMap::new(),
            voxel_size: 0.5,
            radius,
            height_min,
            height_max,
            last_observer: Vec3::ZERO,
            wanted: std::collections::HashSet::new(),
        }
    }

    /// 世界坐标 → ChunkId
    pub fn world_to_chunk(&self, pos: Vec3) -> ChunkId {
        let c = |v: f32| (v / 15.0).floor() as i32;
        ChunkId::new(c(pos.x), c(pos.y).clamp(-2, 2), c(pos.z))
    }

    /// 判定 chunk 8 角点是否有表面穿过
    pub fn chunk_has_surface(min: Vec3, _grid_size: u32, voxel_size: f32) -> bool {
        let max = min + Vec3::splat(30.0 * voxel_size);
        let mut first_sign = None;
        for xi in 0..=1 {
            for yi in 0..=1 {
                for zi in 0..=1 {
                    let p = Vec3::new(
                        if xi == 0 { min.x } else { max.x },
                        if yi == 0 { min.y } else { max.y },
                        if zi == 0 { min.z } else { max.z },
                    );
                    let density = p.y;
                    let sign = density >= 0.0;
                    match first_sign {
                        None => first_sign = Some(sign),
                        Some(s) if s != sign => return true,
                        _ => {}
                    }
                }
            }
        }
        false
    }

    /// 根据观察者位置更新 ChunkManager，填充 wanted 到 ChunkLoadRequest
    pub fn update_for_observer(&mut self, observer: Vec3, req: &mut ChunkLoadRequest) {
        let center = self.world_to_chunk(observer);
        self.wanted.clear();

        let chunk_radius = (self.radius / 15.0).ceil() as i32;
        let y_start = (self.height_min / 15.0).floor() as i32;
        let y_end = (self.height_max / 15.0).ceil() as i32;

        for dx in -chunk_radius..=chunk_radius {
            for dz in -chunk_radius..=chunk_radius {
                for dy in y_start..=y_end {
                    let cid = ChunkId::new(center.x + dx, dy, center.z + dz);
                    self.wanted.insert(cid);
                }
            }
        }

        req.wanted.clear();
        req.wanted.extend(self.wanted.iter().copied());
        self.last_observer = observer;
    }

    /// 计算邻居掩码：bit 0=+X, 1=-X, 2=+Y, 3=-Y, 4=+Z, 5=-Z
    pub fn neighbor_mask(&self, cid: &ChunkId) -> u32 {
        let neighbors = [
            (cid.offset(1, 0, 0), 0u32),   // +X
            (cid.offset(-1, 0, 0), 1u32),  // -X
            (cid.offset(0, 1, 0), 2u32),   // +Y
            (cid.offset(0, -1, 0), 3u32),  // -Y
            (cid.offset(0, 0, 1), 4u32),   // +Z
            (cid.offset(0, 0, -1), 5u32),  // -Z
        ];
        let mut mask = 0u32;
        for (nbr, bit) in &neighbors {
            if self.active.contains_key(nbr) {
                mask |= 1u32 << bit;
            }
        }
        mask
    }
}
