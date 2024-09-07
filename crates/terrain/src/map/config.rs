use std::path::PathBuf;

use std::ops::Range;

use bevy::render::render_resource::ShaderType;
use bevy::render::Extract;
use bevy::{prelude::*, render::extract_resource::ExtractResource};
use project::project_saved_root_path;
use serde::{Deserialize, Serialize};
use settings::Setting;

use crate::setting::TerrainSetting;

#[derive(
    Setting, Resource, Debug, Clone, Serialize, Deserialize, TypePath, Asset, ExtractResource,
)]
pub struct TerrainMapSetting {
    pub grid_num: usize,
    pub grid_cell_size: f64,

    pub rand_point_num: usize,
    // (0.0 ~ 1.0)
    pub rand_point_range_percent: Range<f32>,
    pub rand_point_radius: Range<usize>,
    pub rand_point_height: Range<f64>,

    // and max precipitation is 1.0 - max_base_humidity;
    pub max_base_humidity: f64,

    // 最大最小温度
    pub temperature_range: Range<f64>,
    pub temperature_altitude_range: Range<f64>,
    // 在 height == 0 的温度
    pub temperature_horizontal_range: Range<f64>,

    pub image_save_path: PathBuf,
}

impl Default for TerrainMapSetting {
    fn default() -> Self {
        // image size is GRID_NUM * GRID_CELL_SIZE
        const GRID_NUM: usize = 256;
        const GRID_CELL_SIZE: f64 = 32.0;

        let saved_root_path = project_saved_root_path();

        TerrainMapSetting {
            grid_num: GRID_NUM,
            grid_cell_size: GRID_CELL_SIZE,
            rand_point_num: 12,
            rand_point_range_percent: 0.2..0.8,
            rand_point_radius: 5..10,
            rand_point_height: 0.5..1.0,
            temperature_range: -40.0..40.0,
            temperature_horizontal_range: -20.0..60.0,
            temperature_altitude_range: -20.0..0.0,
            max_base_humidity: 0.3,
            image_save_path: saved_root_path.join("maps"),
        }
    }
}

impl TerrainMapSetting {
    pub fn get_map_size(&self) -> f32 {
        self.grid_num as f32 * self.grid_cell_size as f32
    }
}

#[derive(Resource)]
pub struct TerrainMapContext {
    pub rng: rand_pcg::Pcg32,
}

impl TerrainMapContext {
    pub fn new(seed: u64) -> Self {
        TerrainMapContext {
            rng: rand_pcg::Pcg32::new(seed, 102934719850918234),
        }
    }
}

#[derive(ShaderType, Resource, Default, Clone, Debug)]
pub struct TerrainMapGpuConfig {
    // 图片的大小
    pub terrain_height: f32,
    // 一个像素代表的地图大小
    pub pixel_size: f32,
    // 最小温度
    pub temperature_min: f32,
    // 最大温度
    pub temperature_max: f32,
}

pub fn extract_terrain_map_config(
    mut terrain_map_gpu_config: ResMut<TerrainMapGpuConfig>,
    terrain_map_config: Extract<Res<TerrainMapSetting>>,
    terrain_setting: Extract<Res<TerrainSetting>>,
) {
    terrain_map_gpu_config.temperature_min = terrain_map_config.temperature_range.start as f32;
    terrain_map_gpu_config.temperature_max = terrain_map_config.temperature_range.end as f32;

    let pixel_num = terrain_map_config.grid_num as u32 * terrain_map_config.grid_cell_size as u32;
    terrain_map_gpu_config.terrain_height = terrain_setting.terrain_max_height;
    terrain_map_gpu_config.pixel_size = terrain_setting.get_terrain_size() / pixel_num as f32;
}
