use std::path::PathBuf;

use std::ops::Range;

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy::render::Extract;

use crate::setting::TerrainSetting;

#[derive(Resource)]
pub struct TerrainMapConfig {
    pub grid_num: usize,
    pub grid_cell_size: f64,

    pub rand_point_num: usize,
    // (0 ~ 1) * grid_num
    pub rand_point_range: Range<usize>,
    pub rand_point_radius: Range<usize>,
    pub rand_point_height: Range<f64>,

    // and max precipitation is 1.0 - max_base_humidity;
    pub max_base_humidity: f64,

    // 最大最小温度
    pub temperature_range: Range<f64>,
    pub temperature_altitude_range: Range<f64>,
    // 在 height == 0 的温度
    pub temperature_horizontal_range: Range<f64>,

    pub rng: rand_pcg::Pcg32,

    pub image_save_path: PathBuf,
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
    terrain_map_config: Extract<Res<TerrainMapConfig>>,
    terrain_setting: Extract<Res<TerrainSetting>>,
) {
    terrain_map_gpu_config.temperature_min = terrain_map_config.temperature_range.start as f32;
    terrain_map_gpu_config.temperature_max = terrain_map_config.temperature_range.end as f32;

    let pixel_num = terrain_map_config.grid_num as u32 * terrain_map_config.grid_cell_size as u32;
    terrain_map_gpu_config.terrain_height = terrain_setting.get_terrain_max_height();
    terrain_map_gpu_config.pixel_size = terrain_setting.get_terrain_size() / pixel_num as f32;
}
