//! Per-chunk 管理：ChunkId、ChunkState、ChunkManager。
//! 32³ voxel per chunk，+1 ghost border → grid_size = 32。
//! 水平 80m 半径，垂直 -64~32。

use bevy::{prelude::*, render::extract_resource::ExtractResource};
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
    /// 世界噪声种子
    pub world_seed: u32,
}

impl ChunkManager {
    /// 创建 chunk 管理器。
    /// `seed` 控制地形和岛屿生成（相同 seed → 完全相同的地图）。
    pub fn new(radius: f32, height_min: f32, height_max: f32, seed: u32) -> Self {
        Self {
            active: HashMap::new(),
            voxel_size: 0.5,
            radius,
            height_min,
            height_max,
            last_observer: Vec3::ZERO,
            wanted: std::collections::HashSet::new(),
            world_seed: seed,
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
            (cid.offset(1, 0, 0), 0u32),  // +X
            (cid.offset(-1, 0, 0), 1u32), // -X
            (cid.offset(0, 1, 0), 2u32),  // +Y
            (cid.offset(0, -1, 0), 3u32), // -Y
            (cid.offset(0, 0, 1), 4u32),  // +Z
            (cid.offset(0, 0, -1), 5u32), // -Z
        ];
        let mut mask = 0u32;
        for (nbr, bit) in &neighbors {
            if self.active.contains_key(nbr) {
                mask |= 1u32 << bit;
            }
        }
        mask
    }

    /// 根据 ChunkId 计算该 chunk 的噪声种子，确保相邻 chunk 边界连续。
    pub fn chunk_seed(&self, cid: &ChunkId) -> u32 {
        self.world_seed
            .wrapping_add((cid.x as u32).wrapping_mul(7919))
            .wrapping_add((cid.z as u32).wrapping_mul(6271))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    fn mgr(radius: f32, hmin: f32, hmax: f32, seed: u32) -> ChunkManager {
        ChunkManager::new(radius, hmin, hmax, seed)
    }

    // ── ChunkId ──

    #[test]
    fn chunk_id_new_and_eq() {
        let a = ChunkId::new(1, -2, 3);
        assert_eq!((a.x, a.y, a.z), (1, -2, 3));
        assert_eq!(a, ChunkId::new(1, -2, 3));
        assert_ne!(a, ChunkId::new(1, -2, 4));
    }

    #[test]
    fn world_min_scales_by_15() {
        assert_eq!(
            ChunkId::new(2, -1, 3).world_min(),
            Vec3::new(30.0, -15.0, 45.0)
        );
        assert_eq!(ChunkId::new(0, 0, 0).world_min(), Vec3::ZERO);
        assert_eq!(
            ChunkId::new(-1, 1, -1).world_min(),
            Vec3::new(-15.0, 15.0, -15.0)
        );
    }

    #[test]
    fn distance_euclidean_times_15() {
        assert_approx(ChunkId::new(0, 0, 0).distance(&ChunkId::new(1, 0, 0)), 15.0);
        assert_approx(ChunkId::new(0, 0, 0).distance(&ChunkId::new(0, 0, 0)), 0.0);
        assert_approx(
            ChunkId::new(0, 0, 0).distance(&ChunkId::new(1, 1, 1)),
            15.0 * 3.0f32.sqrt(),
        );
        // 对称性
        assert_approx(
            ChunkId::new(1, 2, 3).distance(&ChunkId::new(4, 5, 6)),
            ChunkId::new(4, 5, 6).distance(&ChunkId::new(1, 2, 3)),
        );
        // 负方向距离为正
        assert_approx(
            ChunkId::new(0, 0, 0).distance(&ChunkId::new(-1, 0, 0)),
            15.0,
        );
    }

    #[test]
    fn offset_adds_chunk_units() {
        assert_eq!(
            ChunkId::new(1, 2, 3).offset(4, -5, 6),
            ChunkId::new(5, -3, 9)
        );
        assert_eq!(ChunkId::new(0, 0, 0).offset(0, 0, 0), ChunkId::new(0, 0, 0));
        assert_eq!(
            ChunkId::new(-1, -1, -1).offset(-1, -1, -1),
            ChunkId::new(-2, -2, -2)
        );
        // 六方向邻居
        assert_eq!(ChunkId::new(0, 0, 0).offset(1, 0, 0), ChunkId::new(1, 0, 0));
        assert_eq!(
            ChunkId::new(0, 0, 0).offset(0, 0, -1),
            ChunkId::new(0, 0, -1)
        );
    }

    // ── ChunkManager ──

    #[test]
    fn new_sets_defaults() {
        let m = mgr(50.0, -50.0, 10.0, 42);
        assert!(m.active.is_empty());
        assert_approx(m.voxel_size, 0.5);
        assert_approx(m.radius, 50.0);
        assert_approx(m.height_min, -50.0);
        assert_approx(m.height_max, 10.0);
        assert_eq!(m.last_observer, Vec3::ZERO);
        assert!(m.wanted.is_empty());
        assert_eq!(m.world_seed, 42);
    }

    #[test]
    fn world_to_chunk_floors_and_clamps_y() {
        let m = mgr(50.0, -50.0, 10.0, 42);
        assert_eq!(
            m.world_to_chunk(Vec3::new(15.0, 15.0, 15.0)),
            ChunkId::new(1, 1, 1)
        );
        assert_eq!(
            m.world_to_chunk(Vec3::new(-15.0, -15.0, -15.0)),
            ChunkId::new(-1, -1, -1)
        );
        assert_eq!(
            m.world_to_chunk(Vec3::new(14.9, 0.0, -15.1)),
            ChunkId::new(0, 0, -2)
        );
        // y 夹取到 [-2, 2]
        assert_eq!(
            m.world_to_chunk(Vec3::new(0.0, 100.0, 0.0)),
            ChunkId::new(0, 2, 0)
        );
        assert_eq!(
            m.world_to_chunk(Vec3::new(0.0, -100.0, 0.0)),
            ChunkId::new(0, -2, 0)
        );
        // x/z 不夹取
        assert_eq!(
            m.world_to_chunk(Vec3::new(10_000.0, 0.0, 0.0)),
            ChunkId::new(666, 0, 0)
        );
        assert_eq!(
            m.world_to_chunk(Vec3::new(0.0, 0.0, -10_000.0)),
            ChunkId::new(0, 0, -667)
        );
    }

    #[test]
    fn chunk_has_surface_crossing_y_plane() {
        let vs = 0.5;
        // min.y < 0 且 max.y >= 0 → 表面穿过
        assert!(ChunkManager::chunk_has_surface(
            Vec3::new(0.0, -1.0, 0.0),
            32,
            vs
        ));
        assert!(ChunkManager::chunk_has_surface(
            Vec3::new(0.0, -0.001, 0.0),
            32,
            vs
        ));
        // 全部 y >= 0 → 无表面（y=0 计为正）
        assert!(!ChunkManager::chunk_has_surface(
            Vec3::new(0.0, 0.0, 0.0),
            32,
            vs
        ));
        assert!(!ChunkManager::chunk_has_surface(
            Vec3::new(0.0, 1.0, 0.0),
            32,
            vs
        ));
        // 全部 y < 0 → 无表面
        assert!(!ChunkManager::chunk_has_surface(
            Vec3::new(0.0, -20.0, 0.0),
            32,
            vs
        ));
        // 边界：max.y 恰好 0（min = -30*vs = -15）→ 角点符号混合（-15 与 0）→ 有表面
        assert!(ChunkManager::chunk_has_surface(
            Vec3::new(0.0, -15.0, 0.0),
            32,
            vs
        ));
        // 不同 voxel_size 缩放
        assert!(ChunkManager::chunk_has_surface(
            Vec3::new(0.0, -1.0, 0.0),
            32,
            1.0
        ));
        assert!(!ChunkManager::chunk_has_surface(
            Vec3::new(0.0, -31.0, 0.0),
            32,
            1.0
        ));
    }

    #[test]
    fn update_for_observer_populates_wanted() {
        let mut m = mgr(50.0, -50.0, 10.0, 42);
        let mut req = ChunkLoadRequest::default();

        m.update_for_observer(Vec3::new(15.0, 0.0, 15.0), &mut req);
        // chunk_radius = ceil(50/15) = 4; dy ∈ [floor(-50/15), ceil(10/15)] = [-4, 1]
        // 数量 = (2*4+1)^2 * 6 = 486
        assert_eq!(m.wanted.len(), 486);
        assert_eq!(req.wanted.len(), 486);
        assert_eq!(m.wanted, req.wanted);
        assert_eq!(m.last_observer, Vec3::new(15.0, 0.0, 15.0));
        assert!(req.wanted.contains(&ChunkId::new(1, 0, 1)));

        // 观察者移动后重新计算
        m.update_for_observer(Vec3::new(1500.0, 0.0, -1500.0), &mut req);
        assert_eq!(m.wanted.len(), 486);
        assert_eq!(req.wanted.len(), 486);
        assert!(req.wanted.contains(&ChunkId::new(100, 0, -100)));
        assert!(!req.wanted.contains(&ChunkId::new(1, 0, 1)));
        assert_eq!(m.last_observer, Vec3::new(1500.0, 0.0, -1500.0));
    }

    #[test]
    fn neighbor_mask_bit_positions() {
        let mut m = mgr(50.0, -50.0, 10.0, 42);
        let c = ChunkId::new(0, 0, 0);
        assert_eq!(m.neighbor_mask(&c), 0);

        m.active.insert(ChunkId::new(1, 0, 0), 0);
        m.active.insert(ChunkId::new(-1, 0, 0), 1);
        m.active.insert(ChunkId::new(0, 1, 0), 2);
        m.active.insert(ChunkId::new(0, -1, 0), 3);
        m.active.insert(ChunkId::new(0, 0, 1), 4);
        m.active.insert(ChunkId::new(0, 0, -1), 5);
        assert_eq!(m.neighbor_mask(&c), 0b11_1111);

        // 移除 +X 邻居 → bit0 清零
        m.active.remove(&ChunkId::new(1, 0, 0));
        assert_eq!(m.neighbor_mask(&c), 0b11_1110);

        // 非邻居不参与
        m.active.insert(ChunkId::new(2, 2, 2), 9);
        assert_eq!(m.neighbor_mask(&c), 0b11_1110);

        // 空 active → 0
        let empty = mgr(50.0, -50.0, 10.0, 42);
        assert_eq!(empty.neighbor_mask(&c), 0);
    }

    #[test]
    fn chunk_seed_deterministic_position_dependent() {
        let m = mgr(50.0, -50.0, 10.0, 42);
        let a = ChunkId::new(3, -2, 7);
        assert_eq!(m.chunk_seed(&a), m.chunk_seed(&a));
        // x/z 不同 → 种子不同
        assert_ne!(m.chunk_seed(&a), m.chunk_seed(&ChunkId::new(4, -2, 7)));
        assert_ne!(m.chunk_seed(&a), m.chunk_seed(&ChunkId::new(3, -2, 8)));
        // y 不参与种子
        assert_eq!(m.chunk_seed(&a), m.chunk_seed(&ChunkId::new(3, 99, 7)));
        // 负坐标与极值不 panic
        let _ = m.chunk_seed(&ChunkId::new(-5, 0, -5));
        let _ = m.chunk_seed(&ChunkId::new(i32::MIN, 0, i32::MAX));
        // world_seed 不同 → 种子不同
        assert_ne!(m.chunk_seed(&a), mgr(50.0, -50.0, 10.0, 7).chunk_seed(&a));
    }
}
