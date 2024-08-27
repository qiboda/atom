use bevy::prelude::*;
use tracing::info;
use voronator::{
    delaunator::{Coord, Vector},
    polygon::Polygon,
    CentroidDiagram,
};

use bevy::math::DVec2;
use bevy::utils::hashbrown::HashMap;

use super::topography::MapTerrainType;

#[derive(Debug, Clone, Copy, Default)]
pub struct SiteInfo {
    pub height: f64,
    // 降雨
    pub precipitation: f64,
    // 基础湿度, 总湿度等于基础湿度加上降雨
    pub base_humidity: f64,
    pub temperature: f64,
    /// area id: 0 to n 等价到随机点的索引, 只对陆地起作用
    pub area_id: usize,
    pub terrain_type: Option<MapTerrainType>,
}

impl SiteInfo {
    pub fn get_total_humidity(&self) -> f64 {
        self.base_humidity + self.precipitation
    }
}

#[derive(Debug, Clone, Resource)]
pub struct TerrainMap {
    pub diagram: CentroidDiagram<MapPoint>,
    pub sites_info: Vec<SiteInfo>,
    pub terrain_types: HashMap<MapTerrainType, Vec<usize>>,
    pub land_random_points: Vec<usize>,
}

pub(crate) fn shared_edge(
    polygon_1: &Polygon<MapPoint>,
    polygon_2: &Polygon<MapPoint>,
) -> Option<[MapPoint; 2]> {
    let mut iter = polygon_1
        .points()
        .iter()
        .filter(|p| polygon_2.points().contains(*p));

    iter.next().and_then(|p| iter.next().map(|p1| [*p, *p1]))
}

impl TerrainMap {
    pub fn new(points: Vec<MapPoint>) -> Self {
        let diagram = CentroidDiagram::new(&points).unwrap();
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
            terrain_types: HashMap::new(),
            land_random_points: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, DerefMut, Deref)]
pub struct MapPoint(pub DVec2);

impl Coord for MapPoint {
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

impl<C> Vector<C> for MapPoint where C: Coord + Clone {}
