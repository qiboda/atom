use std::path::PathBuf;

use std::ops::RangeInclusive;

use atom_core::paths::ProjectPaths;
use bevy::render::Extract;
use bevy::render::render_resource::ShaderType;
use bevy::{prelude::*, render::extract_resource::ExtractResource};
use serde::{Deserialize, Serialize};

use crate::biomes::types::BiomeType;
use crate::terrain::setting::TerrainSetting;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainAreaSetting {
    pub biome_type: BiomeType,
    // 随机岛屿的数量
    pub rand_area_num: RangeInclusive<usize>,
    // 随机岛屿生成位置的范围，使用百分比表示, 范围为0.0到1.0
    pub range_area_position_range: RangeInclusive<Vec2>,
    // 用于随机一个岛屿的半径。
    pub rand_area_radius: RangeInclusive<usize>,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, TypePath, Asset, ExtractResource)]
pub struct TerrainRegionGeneratorSetting {
    // 每个格子的大小，单位为米
    pub grid_cell_size: f32,
    // 要求范围在0.0到1.0之间
    pub point_jitter_range: RangeInclusive<f32>,

    // 超出部分是海洋
    pub area_range: RangeInclusive<f32>,

    // 是一个vector，可以更好的控制不同尺寸的Area的分布情况和大小等。
    pub rand_area_setting: Vec<TerrainAreaSetting>,

    // 图片中每个像素代表的实际地型大小
    pub size_per_pixel: f32,
    pub image_save_path: PathBuf,
}

impl Default for TerrainRegionGeneratorSetting {
    fn default() -> Self {
        const GRID_CELL_SIZE: f32 = 32.0;

        let saved_root_path = ProjectPaths::saved_path();
        let image_save_path = saved_root_path.join("maps");
        // 创建保存目录
        std::fs::create_dir_all(&image_save_path).ok();

        TerrainRegionGeneratorSetting {
            grid_cell_size: GRID_CELL_SIZE,
            point_jitter_range: 0.2..=0.8,
            area_range: 0.2..=0.8,
            // 没有的部分是海洋
            rand_area_setting: vec![
                TerrainAreaSetting {
                    biome_type: BiomeType::Forest,
                    range_area_position_range: Vec2::new(0.2, 0.2)..=Vec2::new(0.8, 0.8),
                    rand_area_radius: 30..=50,
                    rand_area_num: 2..=4,
                },
                TerrainAreaSetting {
                    biome_type: BiomeType::Desert,
                    range_area_position_range: Vec2::new(0.2, 0.2)..=Vec2::new(0.8, 0.8),
                    rand_area_radius: 30..=50,
                    rand_area_num: 1..=1,
                },
                TerrainAreaSetting {
                    biome_type: BiomeType::Plains,
                    range_area_position_range: Vec2::new(0.2, 0.2)..=Vec2::new(0.8, 0.8),
                    rand_area_radius: 30..=50,
                    rand_area_num: 2..=4,
                },
                TerrainAreaSetting {
                    biome_type: BiomeType::Mountains,
                    range_area_position_range: Vec2::new(0.2, 0.2)..=Vec2::new(0.8, 0.8),
                    rand_area_radius: 30..=50,
                    rand_area_num: 1..=2,
                },
                TerrainAreaSetting {
                    biome_type: BiomeType::Swamp,
                    range_area_position_range: Vec2::new(0.2, 0.2)..=Vec2::new(0.8, 0.8),
                    rand_area_radius: 30..=50,
                    rand_area_num: 1..=2,
                },
            ],
            image_save_path,
            size_per_pixel: 2.0,
        }
    }
}

impl TerrainRegionGeneratorSetting {
    pub fn get_grid_num(&self, terrain_size: f32) -> usize {
        assert!(terrain_size % self.grid_cell_size == 0.0);
        (terrain_size / self.grid_cell_size) as usize
    }
}

#[derive(ShaderType, Resource, Default, Clone, Debug)]
pub struct TerrainRegionGpuConfig {
    // 一个像素代表的地图大小
    pub size_per_pixel: f32,
}

pub fn extract_terrain_map_config(
    mut terrain_map_gpu_config: ResMut<TerrainRegionGpuConfig>,
    terrain_map_config: Extract<Res<TerrainRegionGeneratorSetting>>,
    _terrain_setting: Extract<Res<TerrainSetting>>,
) {
    terrain_map_gpu_config.size_per_pixel = terrain_map_config.size_per_pixel;
}
