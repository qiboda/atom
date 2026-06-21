//! Per-chunk 管理：ChunkId、ChunkState、ChunkManager。
//!
//! 32³ voxel per chunk，+1 ghost border → grid_size = 33。
//! 水平 64m 半径，垂直 -32~32。

use bevy::prelude::*;
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

    /// Chunk 原点世界坐标（ghost border 不计入，原点在真实 32³ 起始处）
    pub fn world_min(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * 16.0,
            self.y as f32 * 16.0,
            self.z as f32 * 16.0,
        )
    }

    /// Chunk 间距（正负 1 在相邻 chunk）
    pub fn distance(&self, other: &ChunkId) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt() * 16.0
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

/// GPU 计算槽位
pub struct ChunkSlot {
    /// 所属 ChunkId
    pub chunk_id: ChunkId,
    /// 当前生命周期状态
    pub state: ChunkState,
    /// 在共享 vertex buffer 中的偏移量（顶点数）
    pub vertex_offset: u32,
    /// 在共享 index buffer 中的偏移量（索引数）
    pub index_offset: u32,
    /// GPU compute 各 pass 是否已提交
    pub passes_submitted: [bool; 6],
    /// 需要 readback（靠近玩家，用于碰撞）
    pub needs_readback: bool,
}

impl ChunkSlot {
    /// 创建新 slot
    pub fn new(chunk_id: ChunkId, vertex_offset: u32, index_offset: u32) -> Self {
        Self {
            chunk_id,
            state: ChunkState::Pending,
            vertex_offset,
            index_offset,
            passes_submitted: [false; 6],
            needs_readback: false,
        }
    }
}

/// 管理多 chunk 生命周期（主世界 Resource）
#[derive(Resource)]
pub struct ChunkManager {
    /// 所有活跃 chunk：ChunkId → slot index
    pub active: HashMap<ChunkId, usize>,
    /// 固定大小的 slot 池
    pub slots: Vec<Option<ChunkSlot>>,
    /// 空闲 slot 索引
    pub free_slots: Vec<usize>,
    /// Voxel 边长
    pub voxel_size: f32,
    /// 水平加载半径（米）
    pub radius: f32,
    /// 地形最低高度
    pub height_min: f32,
    /// 地形最高高度
    pub height_max: f32,
    /// 最近一次更新的观察者位置
    last_observer: Vec3,
}

impl ChunkManager {
    /// 创建 chunk 管理器，预分配 `max_chunks` 个 slot
    pub fn new(max_chunks: usize, radius: f32, height_min: f32, height_max: f32) -> Self {
        let mut free_slots = Vec::with_capacity(max_chunks);
        for i in 0..max_chunks {
            free_slots.push(i);
        }
        Self {
            active: HashMap::new(),
            slots: (0..max_chunks).map(|_| None).collect(),
            free_slots,
            voxel_size: 0.5,
            radius,
            height_min,
            height_max,
            last_observer: Vec3::ZERO,
        }
    }

    /// 世界坐标 → ChunkId
    pub fn world_to_chunk(&self, pos: Vec3) -> ChunkId {
        let c = |v: f32| (v / 16.0).floor() as i32;
        ChunkId::new(
            c(pos.x),
            c(pos.y).clamp(-2, 2),
            c(pos.z),
        )
    }

    /// 判定 chunk 是否可能穿过地形表面
    pub fn chunk_has_surface(min: Vec3, _grid_size: u32, voxel_size: f32) -> bool {
        let max = min + Vec3::splat(32.0 * voxel_size);
        let mut first_sign = None;
        for xi in 0..=1 {
            for yi in 0..=1 {
                for zi in 0..=1 {
                    let p = Vec3::new(
                        if xi == 0 { min.x } else { max.x },
                        if yi == 0 { min.y } else { max.y },
                        if zi == 0 { min.z } else { max.z },
                    );
                    // 简化密度测试：y - height_at(x,z) ≈ y
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

    /// 根据观察者位置计算加载/卸载列表
    pub fn update_for_observer(&mut self, observer: Vec3) -> (Vec<ChunkId>, Vec<ChunkId>) {
        let center = self.world_to_chunk(observer);
        let mut wanted = std::collections::HashSet::new();
        let chunk_radius = (self.radius / 16.0).ceil() as i32;
        let y_start = (self.height_min / 16.0).floor() as i32;
        let y_end = (self.height_max / 16.0).ceil() as i32;

        for dx in -chunk_radius..=chunk_radius {
            for dz in -chunk_radius..=chunk_radius {
                let dist = ((dx * dx + dz * dz) as f64).sqrt() * 16.0;
                if dist > self.radius as f64 {
                    continue;
                }
                for dy in y_start..=y_end {
                    let cid = ChunkId::new(center.x + dx, dy, center.z + dz);
                    let min = cid.world_min();
                    if Self::chunk_has_surface(min, 33, self.voxel_size) {
                        wanted.insert(cid);
                    }
                }
            }
        }

        let mut to_load = Vec::new();
        let mut to_unload = Vec::new();
        for (cid, _) in &self.active {
            if !wanted.contains(cid) {
                to_unload.push(*cid);
            }
        }
        for cid in &wanted {
            if !self.active.contains_key(cid) {
                to_load.push(*cid);
            }
        }
        self.last_observer = observer;
        (to_load, to_unload)
    }

    /// 分配 slot 并标记为加载
    pub fn load_chunk(&mut self, chunk_id: ChunkId) -> Option<usize> {
        if self.active.contains_key(&chunk_id) {
            return None;
        }
        let slot_idx = self.free_slots.pop()?;
        self.active.insert(chunk_id, slot_idx);
        self.slots[slot_idx] = Some(ChunkSlot::new(chunk_id, 0, 0));
        Some(slot_idx)
    }

    /// 卸载 chunk，释放 slot
    pub fn unload_chunk(&mut self, chunk_id: &ChunkId) {
        if let Some(slot_idx) = self.active.remove(chunk_id) {
            self.slots[slot_idx] = None;
            self.free_slots.push(slot_idx);
        }
    }

    /// 获取 slot
    pub fn slot(&self, chunk_id: &ChunkId) -> Option<&ChunkSlot> {
        self.active
            .get(chunk_id)
            .and_then(|&idx| self.slots[idx].as_ref())
    }

    /// 获取 slot（可变）
    pub fn slot_mut(&mut self, chunk_id: &ChunkId) -> Option<&mut ChunkSlot> {
        self.active
            .get(chunk_id)
            .and_then(|&idx| self.slots[idx].as_mut())
    }
}
