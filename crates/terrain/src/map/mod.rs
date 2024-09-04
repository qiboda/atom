pub mod compute_height;
pub mod config;

use std::{f64::consts::PI, ops::Not};

use atom_internal::app_state::AppState;
use atom_utils::{
    math::{points_in_triangle, triangle_interpolation},
    swap_data::{SwapData, SwapDataTakeTrait, SwapDataTrait},
};
use bevy::{
    app::Plugin,
    math::{DVec2, Rot2},
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssetUsages,
        texture::ImageSampler,
        RenderApp,
    },
    tasks::{AsyncComputeTaskPool, ParallelSliceMut},
    utils::hashbrown::HashSet,
};
use config::{extract_terrain_map_config, TerrainMapGpuConfig};
use image::{ImageBuffer, Luma};
use imageproc::drawing::{draw_line_segment_mut, draw_polygon_mut};
use map_diagram::{shared_edge, MapPoint, TerrainMap};
use project::project_saved_root_path;
use rand::Rng;
use topography::{
    MapFlatTerrainType, MapHillsLandform, MapMountainLandform, MapPlainLandform, MapTerrainType,
};
use voronator::delaunator::Coord;
use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::TerrainState;

pub mod map_diagram;
pub mod topography;

/// 是否应该区分降雨和湿度。
/// 生成一个高度图。用于过渡混合。
/// 生成一个湿度图。模糊湿度图，避免边缘过于整洁。
/// 生成一个温度图。扭曲温度图，避免过于规整
/// 这三个图合并成一张。减少上传的次数。
///
/// 生成一个生态图。用于确定地形。(应该在gpu生成)
#[derive(Resource, Default, Debug, ExtractResource, Clone)]
pub struct TerrainInfoMap {
    /// r channel: height
    /// g channel: humidity
    /// b channel: temperature
    pub height_climate_map: Handle<Image>,
    // 4个channel，每个通道(u8)表示1层地形类型的占比，这个vec的所有通道总和为255。
    pub biome_map: Handle<Image>,
    pub biome_blend_map: Handle<Image>,
}

#[derive(Default)]
pub struct TerrainMapPlugin;

impl Plugin for TerrainMapPlugin {
    fn build(&self, app: &mut App) {
        // image size is GRID_NUM * GRID_CELL_SIZE
        const GRID_NUM: usize = 256;
        const GRID_CELL_SIZE: f64 = 32.0;

        let rng = rand_pcg::Pcg32::new(10020349, 102934719850918234);

        let saved_root_path = project_saved_root_path();

        app.insert_resource(config::TerrainMapConfig {
            grid_num: GRID_NUM,
            grid_cell_size: GRID_CELL_SIZE,
            rng,
            rand_point_num: 12,
            rand_point_range: (GRID_NUM as f32 * 0.2) as usize..(GRID_NUM as f32 * 0.8) as usize,
            rand_point_radius: 5..10,
            rand_point_height: 0.5..1.0,
            temperature_range: -40.0..40.0,
            temperature_horizontal_range: -20.0..60.0,
            temperature_altitude_range: -20.0..0.0,
            max_base_humidity: 0.3,
            image_save_path: saved_root_path.join("maps"),
        })
        .insert_resource(TerrainInfoMap::default())
        .add_plugins(ExtractResourcePlugin::<TerrainInfoMap>::default())
        .add_systems(
            Update,
            (
                create_terrain_map,
                generate_heights,
                determine_terrain_type_by_height,
                (
                    amount_of_precipitation,
                    generate_temperature,
                    generate_base_humidity,
                ),
                determine_landform,
                (
                    generate_map_image,
                    generate_biome_image,
                    draw_terrain_image,
                    draw_precipitation_image,
                    draw_base_humidity_image,
                    draw_total_humidity_image,
                    draw_temperature_image,
                    draw_delaunay_triangle_image,
                ),
                to_generate_height_map,
            )
                .chain()
                .run_if(
                    in_state(AppState::AppRunning)
                        .and_then(in_state(TerrainState::GenerateTerrainInfoMap)),
                ),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<TerrainMapGpuConfig>()
            .add_systems(ExtractSchedule, extract_terrain_map_config);
    }
}

pub fn create_terrain_map(
    mut commands: Commands,
    mut map_config: ResMut<config::TerrainMapConfig>,
) {
    let grid_num = map_config.grid_num;
    let grid_cell_size = map_config.grid_cell_size;

    let mut points = Vec::with_capacity(map_config.grid_num * map_config.grid_num);

    {
        let _span = info_span!("spawn_points").entered();

        for i in 0..grid_num {
            for j in 0..grid_num {
                let jitter_x = map_config.rng.gen_range(0.2..0.8) * grid_cell_size;
                let jitter_y = map_config.rng.gen_range(0.2..0.8) * grid_cell_size;
                let p = MapPoint::from_xy(
                    i as f64 * grid_cell_size + jitter_x,
                    j as f64 * grid_cell_size + jitter_y,
                );
                points.push(p);
            }
        }
    }

    {
        let _span = info_span!("new terrain map").entered();

        let map = TerrainMap::new(points);
        commands.insert_resource(map)
    }
}

pub fn generate_heights(
    mut map: ResMut<TerrainMap>,
    mut map_config: ResMut<config::TerrainMapConfig>,
) {
    let mut parent_sites = vec![];
    let mut height_declines = vec![];
    for i in 0..map_config.rand_point_num {
        loop {
            let rng_range = map_config.rand_point_range.clone();
            let x = map_config.rng.gen_range(rng_range.clone());
            let y = map_config.rng.gen_range(rng_range);
            // TODO 统一列主序还是行主序
            let index = x * map_config.grid_num + y;

            if parent_sites.contains(&index) {
                continue;
            }

            let height_rng_range = map_config.rand_point_height.clone();
            let h = map_config.rng.gen_range(height_rng_range);
            map.sites_info[index].height = h;
            map.sites_info[index].area_id = i;

            map.land_random_points.push(index);
            parent_sites.push(index);

            let radius_rng_range = map_config.rand_point_radius.clone();
            let r = map_config.rng.gen_range(radius_rng_range);
            height_declines.push(1.0 / r as f64);

            break;
        }
    }

    for height in height_declines.iter() {
        info!("height : {}", height);
    }

    info!("random point over");

    let mut count = 0;
    while !parent_sites.is_empty() {
        let mut children = vec![];
        for parent_index in parent_sites.iter() {
            let parent_site_info = map.sites_info[*parent_index];
            if parent_site_info.height < 0.1 {
                continue;
            }

            let height_decline = height_declines[parent_site_info.area_id];

            let height_adjust = map_config.rng.gen_range(
                (-0.1 + 0.006 * count as f64).min(-0.02)..(0.1 - 0.006 * count as f64).max(0.01),
            );
            for neighbor_index in map.diagram.neighbors[*parent_index].clone() {
                let neighbor_site_info = &mut map.sites_info[neighbor_index];
                if neighbor_site_info.height != 0.0 {
                    continue;
                }
                neighbor_site_info.height =
                    (parent_site_info.height * (1.0 - height_decline) + height_adjust).min(1.0);
                neighbor_site_info.area_id = parent_site_info.area_id;
                children.push(neighbor_index);
            }
        }
        info!("children num: {}", children.len());
        parent_sites = children;
        count += 1;
    }

    info!("generate height over");
}

pub fn determine_landform(mut map: ResMut<TerrainMap>) {
    info!("determine_landform start");
    {
        let plain_cells = map
            .terrain_types
            .remove(&MapTerrainType::Plain(MapPlainLandform::GrassLand))
            .unwrap();

        let mut plain_swamp = vec![];
        let mut plain_desert = vec![];
        let mut plain_rain_forest = vec![];
        let mut plain_forest = vec![];
        let mut plain_grass_land = vec![];
        let mut plain_snow = vec![];
        let mut plain_ice = vec![];
        for cell in plain_cells.iter() {
            let temperature = map.sites_info[*cell].temperature;
            let total_humidity =
                map.sites_info[*cell].precipitation + map.sites_info[*cell].base_humidity;
            match MapPlainLandform::determine_landform(temperature, total_humidity) {
                MapPlainLandform::Swamp => {
                    plain_swamp.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::Swamp));
                }
                MapPlainLandform::Desert => {
                    plain_desert.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::Desert));
                }
                MapPlainLandform::RainForest => {
                    plain_rain_forest.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::RainForest));
                }
                MapPlainLandform::Forest => {
                    plain_forest.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::Forest));
                }
                MapPlainLandform::GrassLand => {
                    plain_grass_land.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::GrassLand));
                }
                MapPlainLandform::Snow => {
                    plain_snow.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::Snow));
                }
                MapPlainLandform::Ice => {
                    plain_ice.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Plain(MapPlainLandform::Ice));
                }
            }
        }

        map.terrain_types
            .insert(MapTerrainType::Plain(MapPlainLandform::Swamp), plain_swamp);
        map.terrain_types.insert(
            MapTerrainType::Plain(MapPlainLandform::Desert),
            plain_desert,
        );
        map.terrain_types.insert(
            MapTerrainType::Plain(MapPlainLandform::RainForest),
            plain_rain_forest,
        );
        map.terrain_types.insert(
            MapTerrainType::Plain(MapPlainLandform::Forest),
            plain_forest,
        );
        map.terrain_types.insert(
            MapTerrainType::Plain(MapPlainLandform::GrassLand),
            plain_grass_land,
        );
        map.terrain_types
            .insert(MapTerrainType::Plain(MapPlainLandform::Snow), plain_snow);
        map.terrain_types
            .insert(MapTerrainType::Plain(MapPlainLandform::Ice), plain_ice);

        info!("determine_landform plain");
    }

    {
        let hills_cells = map
            .terrain_types
            .remove(&MapTerrainType::Hills(MapHillsLandform::GrassLand))
            .unwrap();

        let mut hills_swamp = vec![];
        let mut hills_desert = vec![];
        let mut hills_rain_forest = vec![];
        let mut hills_forest = vec![];
        let mut hills_grass_land = vec![];
        let mut hills_snow = vec![];
        let mut hills_ice = vec![];

        for cell in hills_cells.iter() {
            let temperature = map.sites_info[*cell].temperature;
            let total_humidity =
                map.sites_info[*cell].precipitation + map.sites_info[*cell].base_humidity;
            match MapHillsLandform::determine_landform(temperature, total_humidity) {
                MapHillsLandform::Swamp => {
                    hills_swamp.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::Swamp));
                }
                MapHillsLandform::Desert => {
                    hills_desert.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::Desert));
                }
                MapHillsLandform::RainForest => {
                    hills_rain_forest.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::RainForest));
                }
                MapHillsLandform::Forest => {
                    hills_forest.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::Forest));
                }
                MapHillsLandform::GrassLand => {
                    hills_grass_land.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::GrassLand));
                }
                MapHillsLandform::Snow => {
                    hills_snow.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::Snow));
                }
                MapHillsLandform::Ice => {
                    hills_ice.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Hills(MapHillsLandform::Ice));
                }
            }
        }

        map.terrain_types
            .insert(MapTerrainType::Hills(MapHillsLandform::Swamp), hills_swamp);
        map.terrain_types.insert(
            MapTerrainType::Hills(MapHillsLandform::Desert),
            hills_desert,
        );
        map.terrain_types.insert(
            MapTerrainType::Hills(MapHillsLandform::RainForest),
            hills_rain_forest,
        );
        map.terrain_types.insert(
            MapTerrainType::Hills(MapHillsLandform::Forest),
            hills_forest,
        );
        map.terrain_types.insert(
            MapTerrainType::Hills(MapHillsLandform::GrassLand),
            hills_grass_land,
        );
        map.terrain_types
            .insert(MapTerrainType::Hills(MapHillsLandform::Snow), hills_snow);
        map.terrain_types
            .insert(MapTerrainType::Hills(MapHillsLandform::Ice), hills_ice);

        info!("determine_landform hills");
    }

    {
        let mountain_cells = map
            .terrain_types
            .remove(&MapTerrainType::Mountain(MapMountainLandform::Common))
            .unwrap();

        let mut mountain_common = vec![];
        let mut mountain_snow = vec![];
        let mut mountain_volcano = vec![];
        for cell in mountain_cells.iter() {
            let temperature = map.sites_info[*cell].temperature;
            let total_humidity =
                map.sites_info[*cell].precipitation + map.sites_info[*cell].base_humidity;
            match MapMountainLandform::determine_landform(temperature, total_humidity) {
                MapMountainLandform::Common => {
                    mountain_common.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Mountain(MapMountainLandform::Common));
                }
                MapMountainLandform::Snow => {
                    mountain_snow.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Mountain(MapMountainLandform::Snow));
                }
                MapMountainLandform::Volcano => {
                    mountain_volcano.push(*cell);
                    map.sites_info[*cell].terrain_type =
                        Some(MapTerrainType::Mountain(MapMountainLandform::Volcano));
                }
            }
        }

        map.terrain_types.insert(
            MapTerrainType::Mountain(MapMountainLandform::Common),
            mountain_common,
        );
        map.terrain_types.insert(
            MapTerrainType::Mountain(MapMountainLandform::Snow),
            mountain_snow,
        );
        map.terrain_types.insert(
            MapTerrainType::Mountain(MapMountainLandform::Volcano),
            mountain_volcano,
        );
    }

    info!("determine_landform mountain");
}

pub fn determine_terrain_type_by_height(mut map: ResMut<TerrainMap>) {
    // ocean terrain type
    let mut oceans_cells = HashSet::new();

    {
        let _span = info_span!("determine ocean terrain type").entered();

        let mut ocean_neighbors = SwapData::<Vec<usize>>::default();
        ocean_neighbors.insert(0);
        ocean_neighbors.swap();

        loop {
            let last = ocean_neighbors.take_last();
            if last.is_empty() {
                break;
            }

            for nei in last {
                if oceans_cells.contains(&nei).not() && map.sites_info[nei].height < 0.1 {
                    oceans_cells.insert(nei);
                    ocean_neighbors.extend(map.diagram.neighbors[nei].clone());
                    map.sites_info[nei].terrain_type = Some(MapTerrainType::Ocean);
                }
            }

            ocean_neighbors.swap();
        }
    }

    info!("generate ocean over");

    let mut plain_cells = vec![];
    let mut hills_cells = vec![];
    let mut mountain_cells = vec![];
    let mut lake_cells = vec![];

    {
        let _span = info_span!("determine lake and island terrain type").entered();
        let mut island_data = SwapData::<Vec<usize>>::default();
        island_data.extend(map.land_random_points.clone());
        island_data.swap();

        loop {
            let last = island_data.take_last();
            if last.is_empty() {
                break;
            }

            for point in last {
                if map.sites_info[point].terrain_type.is_some() {
                    continue;
                }

                match map.sites_info[point].height {
                    0.0..0.1 => {
                        lake_cells.push(point);
                        map.sites_info[point].terrain_type = Some(MapTerrainType::Lake);
                    }
                    0.1..0.2 => {
                        plain_cells.push(point);
                        map.sites_info[point].terrain_type =
                            Some(MapTerrainType::Plain(MapPlainLandform::GrassLand));
                    }
                    0.2..0.5 => {
                        hills_cells.push(point);
                        map.sites_info[point].terrain_type =
                            Some(MapTerrainType::Hills(MapHillsLandform::GrassLand));
                    }
                    0.5..=1.0 => {
                        mountain_cells.push(point);
                        map.sites_info[point].terrain_type =
                            Some(MapTerrainType::Mountain(MapMountainLandform::Common));
                    }
                    _ => {
                        panic!("height out of range");
                    }
                }

                island_data.extend(map.diagram.neighbors[point].clone());
            }

            island_data.swap();
        }
    }

    info!("generate lake over");

    let mut beach_cells = HashSet::new();

    {
        let _span = info_span!("determine beach terrain type").entered();

        for island in plain_cells.iter() {
            let neighbors = &map.diagram.neighbors[*island];
            if neighbors.iter().any(|x| oceans_cells.contains(x)) {
                beach_cells.insert(*island);
                map.sites_info[*island].terrain_type = Some(MapTerrainType::Beach);
            }
        }
        plain_cells.retain(|x| !beach_cells.contains(x));

        for island in hills_cells.iter() {
            let neighbors = &map.diagram.neighbors[*island];
            if neighbors.iter().any(|x| oceans_cells.contains(x)) {
                beach_cells.insert(*island);
                map.sites_info[*island].terrain_type = Some(MapTerrainType::Beach);
            }
        }
        hills_cells.retain(|x| !beach_cells.contains(x));

        for island in mountain_cells.iter() {
            let neighbors = &map.diagram.neighbors[*island];
            if neighbors.iter().any(|x| oceans_cells.contains(x)) {
                beach_cells.insert(*island);
                map.sites_info[*island].terrain_type = Some(MapTerrainType::Beach);
            }
        }
        mountain_cells.retain(|x| !beach_cells.contains(x));
    }

    {
        let _span = info_span!("determine terrain type insert data").entered();

        info!(
            "terrain type num: island: {}, hills: {}, mountain: {}, lake: {}, beach: {}, ocean: {}",
            plain_cells.len(),
            hills_cells.len(),
            mountain_cells.len(),
            lake_cells.len(),
            beach_cells.len(),
            oceans_cells.len()
        );

        map.terrain_types.insert(
            MapTerrainType::Plain(MapPlainLandform::GrassLand),
            plain_cells,
        );
        map.terrain_types.insert(
            MapTerrainType::Hills(MapHillsLandform::GrassLand),
            hills_cells,
        );
        map.terrain_types.insert(
            MapTerrainType::Mountain(MapMountainLandform::Common),
            mountain_cells,
        );
        map.terrain_types.insert(MapTerrainType::Lake, lake_cells);
        map.terrain_types
            .insert(MapTerrainType::Beach, beach_cells.into_iter().collect());
        map.terrain_types
            .insert(MapTerrainType::Ocean, oceans_cells.into_iter().collect());
    }

    info!("generate beach over");
}

pub fn generate_temperature(
    mut map: ResMut<TerrainMap>,
    map_config: Res<config::TerrainMapConfig>,
) {
    let temperature_range = map_config.temperature_range.clone();
    let temperature_altitude_range = map_config.temperature_altitude_range.clone();
    let temperature_horizontal_range = map_config.temperature_horizontal_range.clone();
    for x in 0..map_config.grid_num {
        let base_temperature = if x * 2 < map_config.grid_num {
            temperature_horizontal_range.start.lerp(
                temperature_horizontal_range.end,
                (x * 2) as f64 / map_config.grid_num as f64,
            )
        } else {
            temperature_horizontal_range.start.lerp(
                temperature_horizontal_range.end,
                ((map_config.grid_num - x) * 2) as f64 / map_config.grid_num as f64,
            )
        };
        for y in 0..map_config.grid_num {
            let index = x * map_config.grid_num + y;
            let altitude_temperature = temperature_altitude_range.start.lerp(
                temperature_altitude_range.end,
                map.sites_info[index].height / 1.0,
            );
            map.sites_info[index].temperature = (base_temperature + altitude_temperature)
                .clamp(temperature_range.start, temperature_range.end - 1.0);
        }
    }
    info!("generate_temperature end");
}

fn select_neighbor_on_dir(
    map: &TerrainMap,
    current: usize,
    neighbors: &[usize],
    dir: DVec2,
) -> [usize; 4] {
    let _span = info_span!("select neighbor on dir").entered();

    let mut selected = [usize::MAX; 4];

    let current_site = map.diagram.sites[current];
    let min_angle = PI * 0.5;
    let mut index = 0;
    for nei in neighbors.iter() {
        let neighbor_site = map.diagram.sites[*nei];
        if map.sites_info[current].terrain_type == Some(MapTerrainType::Beach)
            && map.sites_info[*nei].terrain_type == Some(MapTerrainType::Beach)
        {
            continue;
        }
        let angle = (neighbor_site.0 - current_site.0)
            .normalize()
            .angle_between(dir.normalize())
            .abs();
        if min_angle > angle {
            selected[index] = *nei;
            index += 1;
            if index >= 4 {
                break;
            }
        }
    }
    selected
}

pub fn generate_base_humidity(
    mut map: ResMut<TerrainMap>,
    map_config: Res<config::TerrainMapConfig>,
) {
    let base_humidity = map_config.max_base_humidity;

    let beach = map
        .terrain_types
        .get(&MapTerrainType::Beach)
        .unwrap()
        .clone();
    let lake = map
        .terrain_types
        .get(&MapTerrainType::Lake)
        .unwrap()
        .clone();

    let mut currents = beach.clone();
    currents.extend(lake);
    while !currents.is_empty() {
        let mut neighbors = vec![];
        for c in currents.iter() {
            if map.sites_info[*c].terrain_type == Some(MapTerrainType::Ocean) {
                continue;
            }

            if map.sites_info[*c].base_humidity > 0.0 {
                continue;
            }

            map.sites_info[*c].base_humidity = base_humidity * (1.1 - map.sites_info[*c].height);

            let ns = map.diagram.neighbors[*c].clone();
            neighbors.extend(ns);
        }

        info!("neighbors: {}", neighbors.len());
        currents = neighbors;
    }
}

pub fn amount_of_precipitation(
    mut map: ResMut<TerrainMap>,
    mut map_config: ResMut<config::TerrainMapConfig>,
) {
    let beach = map
        .terrain_types
        .get(&MapTerrainType::Beach)
        .unwrap()
        .clone();

    let degree = map_config.rng.gen_range(0.0..360.0);
    let wind_dir = Rot2::degrees(degree);
    let wind_dir = wind_dir.normalize();
    let dir = DVec2::new(wind_dir.cos as f64, wind_dir.sin as f64);

    let default_precipitation = 1.0 - map_config.max_base_humidity;

    let mut clouds = SwapData::<Vec<usize>>::default();
    clouds.extend(beach.clone());
    clouds.swap();

    let mut clouds_precipitation = default_precipitation;

    let mut next = HashSet::new();
    loop {
        let last = clouds.take_last();
        if last.is_empty() {
            break;
        }

        let mut total_clouds_precipitation = 0.0;
        let mut count = 0.0;
        for b in last {
            if map.sites_info[b].terrain_type == Some(MapTerrainType::Ocean) {
                continue;
            }

            if map.sites_info[b].precipitation > 0.0 {
                continue;
            }

            map.sites_info[b].precipitation = clouds_precipitation * map.sites_info[b].height;
            total_clouds_precipitation += clouds_precipitation * (1.0 - map.sites_info[b].height);
            count += 1.0;

            if map.sites_info[b].height > 0.9 {
                continue;
            }

            let neighbors = &map.diagram.neighbors[b];
            next.extend(select_neighbor_on_dir(&map, b, neighbors, dir));
        }

        clouds_precipitation = total_clouds_precipitation / count;

        next.remove(&usize::MAX);
        info!("precipitation: next size: {}", next.len());
        clouds.extend(next.iter().copied().collect());
        clouds.swap();
        next.clear();
    }

    info!("amount_of_precipitation seconds");
}

pub fn generate_map_image(
    map: Res<TerrainMap>,
    map_config: Res<config::TerrainMapConfig>,
    mut map_images: ResMut<TerrainInfoMap>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut terrain_map_image = image::Rgb32FImage::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    let mut terrain_height_image = Some(ImageBuffer::<Luma<u16>, Vec<u16>>::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    ));
    let mut terrain_humidity_image = Some(ImageBuffer::<Luma<u16>, Vec<u16>>::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    ));
    let mut terrain_temperature_image = Some(ImageBuffer::<Luma<u16>, Vec<u16>>::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    ));

    for triangle_indices in map.diagram.delaunay.triangles.chunks_exact(3) {
        let pt0 = map.diagram.sites[triangle_indices[0]].0;
        let pt1 = map.diagram.sites[triangle_indices[1]].0;
        let pt2 = map.diagram.sites[triangle_indices[2]].0;

        let points = points_in_triangle(pt0.as_vec2(), pt1.as_vec2(), pt2.as_vec2());

        let h0 = map.sites_info[triangle_indices[0]].height;
        let h1 = map.sites_info[triangle_indices[1]].height;
        let h2 = map.sites_info[triangle_indices[2]].height;

        let hu0 = map.sites_info[triangle_indices[0]].get_total_humidity();
        let hu1 = map.sites_info[triangle_indices[1]].get_total_humidity();
        let hu2 = map.sites_info[triangle_indices[2]].get_total_humidity();

        let t0 = map.sites_info[triangle_indices[0]].temperature;
        let t1 = map.sites_info[triangle_indices[1]].temperature;
        let t2 = map.sites_info[triangle_indices[2]].temperature;

        for point in points {
            let height = triangle_interpolation(
                point.as_vec2(),
                pt0.as_vec2(),
                pt1.as_vec2(),
                pt2.as_vec2(),
                h0 as f32,
                h1 as f32,
                h2 as f32,
            );
            let humidity = triangle_interpolation(
                point.as_vec2(),
                pt0.as_vec2(),
                pt1.as_vec2(),
                pt2.as_vec2(),
                hu0 as f32,
                hu1 as f32,
                hu2 as f32,
            );
            let temperature = triangle_interpolation(
                point.as_vec2(),
                pt0.as_vec2(),
                pt1.as_vec2(),
                pt2.as_vec2(),
                t0 as f32,
                t1 as f32,
                t2 as f32,
            );

            terrain_map_image.put_pixel(
                point.x,
                point.y,
                image::Rgb([height, humidity, temperature]),
            );

            terrain_height_image.as_mut().unwrap().put_pixel(
                point.x,
                point.y,
                image::Luma([(height * 65535.0) as u16]),
            );

            terrain_humidity_image.as_mut().unwrap().put_pixel(
                point.x,
                point.y,
                image::Luma([(humidity * 65535.0) as u16]),
            );

            let temperature_percent = (temperature as f64 - map_config.temperature_range.start)
                / (map_config.temperature_range.end - map_config.temperature_range.start);

            terrain_temperature_image.as_mut().unwrap().put_pixel(
                point.x,
                point.y,
                image::Luma([(temperature_percent * 65535.0) as u16]),
            );
        }
    }

    let image = Image::from_dynamic(
        image::DynamicImage::ImageRgb32F(terrain_map_image),
        false,
        RenderAssetUsages::RENDER_WORLD,
    );

    map_images.height_climate_map = images.add(image);

    if let Some(x) = terrain_height_image.as_ref() {
        x.save(map_config.image_save_path.join("terrain map height.png"))
            .unwrap();
    }

    if let Some(x) = terrain_humidity_image.as_ref() {
        x.save(map_config.image_save_path.join("terrain map humidity.png"))
            .unwrap();
    }

    if let Some(x) = terrain_temperature_image.as_ref() {
        x.save(
            map_config
                .image_save_path
                .join("terrain map temperature.png"),
        )
        .unwrap();
    }

    info!("generate map image");
}

pub fn generate_biome_image(
    map: Res<TerrainMap>,
    map_config: Res<config::TerrainMapConfig>,
    mut map_images: ResMut<TerrainInfoMap>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("generate map start");
    let width = map_config.grid_num as u32 * map_config.grid_cell_size as u32;
    let height = map_config.grid_num as u32 * map_config.grid_cell_size as u32;

    let biome_num = MapFlatTerrainType::MAX;
    let image_num = (biome_num + 3) / 4;
    let mut biome_blend_image_vec = Vec::with_capacity(image_num * 4);
    for _ in 0..(image_num * 4) {
        let image = image::GrayImage::new(width, height);
        biome_blend_image_vec.push(image);
    }

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let terrain_type = map.sites_info[i].terrain_type.unwrap();
        let flat_terrain_type: MapFlatTerrainType = terrain_type.into();
        let biome_color = image::Luma([u8::MAX]);
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 {
            let biome_image = &mut biome_blend_image_vec[flat_terrain_type as usize];
            draw_polygon_mut(biome_image, points.as_slice(), biome_color);
        }
    }

    info!("generate map center");

    let mut rgba_image = image::RgbaImage::new(width, height * image_num as u32);

    let thread_pool = AsyncComputeTaskPool::get();
    let mut flat_sample = rgba_image.as_flat_samples_mut();
    let mut image_slice = flat_sample.as_mut_slice();
    image_slice.par_chunk_map_mut(
        thread_pool,
        (width * height * 4) as usize,
        |index, chunk| {
            info!("par chunk map chunk: {}", index);
            for x in 0..width {
                for y in 0..height {
                    // 得到所有的gray。
                    let r = biome_blend_image_vec[index * 4].get_pixel(x, y).0[0];
                    let g = biome_blend_image_vec[index * 4 + 1].get_pixel(x, y).0[0];
                    let b = biome_blend_image_vec[index * 4 + 2].get_pixel(x, y).0[0];
                    let a = biome_blend_image_vec[index * 4 + 3].get_pixel(x, y).0[0];
                    chunk[((x * height + y) * 4) as usize] = r;
                    chunk[((x * height + y) * 4 + 1) as usize] = g;
                    chunk[((x * height + y) * 4 + 2) as usize] = b;
                    chunk[((x * height + y) * 4 + 3) as usize] = a;
                }
            }
        },
    );

    // info!("generate map value normalize");

    rgba_image
        .save(map_config.image_save_path.join("biome_array.png"))
        .unwrap();

    // get rgba 8 unsigned norm format line interpolation
    let mut biome_render_image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: image_num as u32,
        },
        TextureDimension::D2,
        rgba_image.into_raw(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    biome_render_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    biome_render_image.sampler = ImageSampler::linear();

    let mut blend_biome_render_image = biome_render_image.clone();
    blend_biome_render_image.texture_descriptor.usage |= TextureUsages::STORAGE_BINDING;
    let handle = images.add(blend_biome_render_image);
    map_images.biome_blend_map = handle;

    let handle = images.add(biome_render_image);
    map_images.biome_map = handle;

    info!("generate map end");
}

pub fn draw_terrain_image(map: Res<TerrainMap>, map_config: Res<config::TerrainMapConfig>) {
    let mut image = image::ImageBuffer::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let terrain_type = map.sites_info[i].terrain_type.unwrap();
        let color: [u8; 4] = terrain_type.get_color();
        let color = image::Rgba(color);
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 {
            draw_polygon_mut(&mut image, points.as_slice(), color);
        }

        let neighbors = map.diagram.neighbors[i].clone();
        for neighbor in neighbors {
            if let Some([p0, p1]) = shared_edge(cell, &map.diagram.cells[neighbor]) {
                if map.sites_info[neighbor]
                    .terrain_type
                    .unwrap()
                    .terrain_type_eq(map.sites_info[i].terrain_type.as_ref().unwrap())
                    .not()
                {
                    draw_line_segment_mut(
                        &mut image,
                        (p0.x as f32, p0.y as f32),
                        (p1.x as f32, p1.y as f32),
                        image::Rgba([255, 0, 0, 255]),
                    );
                }
            }
        }
    }

    let colors = MapTerrainType::get_all_color();
    let x = 5;
    let mut y = 5;
    for color in colors {
        let color = image::Rgba(color);

        let index = x * map_config.grid_num + y;
        let cell = &map.diagram.cells[index];
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        draw_polygon_mut(&mut image, points.as_slice(), color);
        y += 2;
    }

    image
        .save(map_config.image_save_path.join("terrain_type.png"))
        .unwrap();
}

pub fn draw_base_humidity_image(map: Res<TerrainMap>, map_config: Res<config::TerrainMapConfig>) {
    let mut image = image::ImageBuffer::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let color = if map.sites_info[i].base_humidity == 0.0 {
            [0, 0, 0, 255]
        } else {
            let base_humidity = map.sites_info[i].base_humidity;
            let b = 100.0.lerp(255.0, base_humidity);
            [0, 0, b as u8, 255]
        };
        let color = image::Rgba(color);
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 {
            draw_polygon_mut(&mut image, points.as_slice(), color);
        }
    }

    image
        .save(map_config.image_save_path.join("base_humidity.png"))
        .unwrap();
}

pub fn draw_total_humidity_image(map: Res<TerrainMap>, map_config: Res<config::TerrainMapConfig>) {
    let mut image = image::ImageBuffer::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let total_humidity = map.sites_info[i].base_humidity + map.sites_info[i].precipitation;
        let color = if total_humidity == 0.0 {
            [0, 0, 0, 255]
        } else {
            let b = 100.0.lerp(255.0, total_humidity);
            [0, 0, b as u8, 255]
        };
        let color = image::Rgba(color);
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 {
            draw_polygon_mut(&mut image, points.as_slice(), color);
        }
    }

    image
        .save(map_config.image_save_path.join("total_humidity.png"))
        .unwrap();
}

pub fn draw_precipitation_image(map: Res<TerrainMap>, map_config: Res<config::TerrainMapConfig>) {
    let mut image = image::ImageBuffer::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let color = if map.sites_info[i].precipitation == 0.0 {
            [0, 0, 0, 255]
        } else {
            let precipitation = map.sites_info[i].precipitation;
            let b = 100.0.lerp(255.0, precipitation);
            [0, 0, b as u8, 255]
        };
        let color = image::Rgba(color);
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 {
            draw_polygon_mut(&mut image, points.as_slice(), color);
            // let font = FontRef::try_from_slice(DEFAULT_FONT_DATA).unwrap();
            // draw_text_mut(&mut image, color, x, y, scale, font, text);
        }
    }

    image
        .save(map_config.image_save_path.join("precipitation.png"))
        .unwrap();
}

pub fn draw_temperature_image(map: Res<TerrainMap>, map_config: Res<config::TerrainMapConfig>) {
    let mut image = image::ImageBuffer::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let temperature = map.sites_info[i].temperature;

        let color: [u8; 4] = match temperature {
            -40.0..-20.0 => [0, 0, 255, 255],
            -20.0..-10.0 => [0, 0, 200, 255],
            -10.0..0.0 => [0, 0, 155, 255],
            0.0..1.0 => [255, 255, 255, 255],
            1.0..20.0 => [155, 0, 0, 255],
            20.0..30.0 => [200, 0, 0, 255],
            30.0..40.0 => [255, 0, 0, 255],
            _ => {
                panic!("error temperature: {}", temperature);
            }
        };

        let color = image::Rgba(color);
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 {
            draw_polygon_mut(&mut image, points.as_slice(), color);
        }
    }

    image
        .save(map_config.image_save_path.join("temperature.png"))
        .unwrap();
}

fn draw_delaunay_triangle_image(map: Res<TerrainMap>, map_config: Res<config::TerrainMapConfig>) {
    let mut image: ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
        map_config.grid_num as u32 * map_config.grid_cell_size as u32,
    );

    for triangle_indices in map.diagram.delaunay.triangles.chunks_exact(3) {
        let pt0 = map.diagram.sites[triangle_indices[0]].0;
        let pt1 = map.diagram.sites[triangle_indices[1]].0;
        let pt2 = map.diagram.sites[triangle_indices[2]].0;

        draw_line_segment_mut(
            &mut image,
            (pt0.x as f32, pt0.y as f32),
            (pt1.x as f32, pt1.y as f32),
            image::Rgba([255, 255, 255, 255]),
        );
        draw_line_segment_mut(
            &mut image,
            (pt0.x as f32, pt0.y as f32),
            (pt2.x as f32, pt2.y as f32),
            image::Rgba([255, 255, 255, 255]),
        );
        draw_line_segment_mut(
            &mut image,
            (pt1.x as f32, pt1.y as f32),
            (pt2.x as f32, pt2.y as f32),
            image::Rgba([255, 255, 255, 255]),
        );
    }

    image
        .save(map_config.image_save_path.join("delaunay.png"))
        .unwrap();
}

fn to_generate_height_map(mut state: ResMut<NextState<TerrainState>>) {
    state.set(TerrainState::GenerateHeightMap);
    info!("to_generate_height_map");
}
