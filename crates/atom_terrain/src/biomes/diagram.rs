use bevy::prelude::*;
use tracing::info;
use voronator::{
    CentroidDiagram,
    delaunator::{Coord, Vector},
    polygon::Polygon,
};

use bevy::math::DVec2;

use crate::biomes::types::BiomeType;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SiteInfo {
    // grid 的 x,y 位置
    pub pos: UVec2,
    /// area id: 0 to n 等价到随机点的索引, 只对陆地起作用，其实就是岛屿的id。
    pub area_id: usize,
    /// 权重，用于Area的展开计算。
    pub area_weight: f32,

    pub biome_type: BiomeType,
}

impl Default for SiteInfo {
    fn default() -> Self {
        Self {
            area_id: usize::MAX,
            area_weight: 0.0,
            biome_type: BiomeType::Ocean,
            pos: UVec2::ZERO,
        }
    }
}

#[derive(Resource)]
pub(crate) struct TerrainRegion {
    // 质心图，他的对偶图是 voronoi 图
    pub diagram: CentroidDiagram<RegionPoint>,
    pub sites_info: Vec<SiteInfo>,
    pub area_random_points: Vec<usize>,
}

pub(crate) fn shared_edge(
    polygon_1: &Polygon<RegionPoint>,
    polygon_2: &Polygon<RegionPoint>,
) -> Option<[RegionPoint; 2]> {
    let mut iter = polygon_1
        .points()
        .iter()
        .filter(|p| polygon_2.points().contains(*p));

    iter.next().and_then(|p| iter.next().map(|p1| [*p, *p1]))
}

impl TerrainRegion {
    pub fn new(points: Vec<RegionPoint>) -> Self {
        let diagram = CentroidDiagram::new(&points).expect("Failed to create CentroidDiagram");
        info!(
            "map site num: {}, center num: {}, cell num: {}, neighbor num: {}",
            diagram.sites.len(),
            diagram.centers.len(),
            diagram.cells.len(),
            diagram.neighbors.len()
        );
        let site_len = diagram.sites.len();
        Self {
            sites_info: vec![SiteInfo::default(); site_len],
            diagram,
            area_random_points: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, DerefMut, Deref)]
pub(crate) struct RegionPoint(pub DVec2);

impl Coord for RegionPoint {
    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn from_xy(x: f64, y: f64) -> Self {
        Self(DVec2 { x, y })
    }
}

impl<C> Vector<C> for RegionPoint where C: Coord + Clone {}
