use super::diagram::{RegionPoint, TerrainRegion, shared_edge};
use bevy::{
    app::Plugin,
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        RenderApp,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
    },
};
use image::ImageBuffer;
use imageproc::drawing::{draw_line_segment_mut, draw_polygon_mut};
use rand::Rng;
use voronator::delaunator::Coord;

use crate::terrain::TerrainState;
use crate::{
    biomes::config::{
        TerrainRegionGeneratorSetting, TerrainRegionGpuConfig, extract_terrain_map_config,
    },
    terrain::setting::TerrainSetting,
};

#[derive(Resource, Default, Debug, ExtractResource, Clone)]
pub struct TerrainRegionInfo {
    pub biome_image: Handle<Image>,
}

#[derive(Resource)]
pub struct TerrainRegionGeneratorContext {
    pub rng: rand_pcg::Pcg32,
}

impl TerrainRegionGeneratorContext {
    pub fn new(seed: u64) -> Self {
        TerrainRegionGeneratorContext {
            rng: rand_pcg::Pcg32::new(seed, 102934719850918234),
        }
    }
}

#[derive(Default)]
pub struct TerrainRegionGeneratorPlugin {
    pub debug: bool,
}

impl Plugin for TerrainRegionGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainRegionGeneratorContext::new(1234))
            .insert_resource(TerrainRegionInfo::default())
            .insert_resource(TerrainRegionGeneratorSetting::default())
            .add_plugins(ExtractResourcePlugin::<TerrainRegionInfo>::default())
            .add_plugins(ExtractResourcePlugin::<TerrainRegionGeneratorSetting>::default())
            .add_systems(
                Update,
                (
                    generate_centroid_diagram,
                    generate_area_data,
                    generate_biome_image,
                    to_generate_terrain_mesh,
                )
                    .chain()
                    .run_if(in_state(TerrainState::GenerateTerrainRegion)),
            );

        if self.debug {
            app.add_systems(
                OnExit(TerrainState::GenerateTerrainRegion),
                (draw_delaunay_triangle_image, draw_area_image).chain(),
            );
        }
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<TerrainRegionGpuConfig>()
            .add_systems(ExtractSchedule, extract_terrain_map_config);
    }
}

fn generate_centroid_diagram(
    mut commands: Commands,
    generator_setting: Res<TerrainRegionGeneratorSetting>,
    mut generator_context: ResMut<TerrainRegionGeneratorContext>,
    terrain_setting: Res<TerrainSetting>,
) {
    let grid_cell_size = generator_setting.grid_cell_size;
    let grid_num = generator_setting.get_grid_num(terrain_setting.get_terrain_size());

    let mut points = Vec::with_capacity(grid_num * grid_num);

    {
        let _span = info_span!("spawn_points").entered();

        // 均匀的切割Grid，之后在每个Grid内随机抖动，选取一个点。
        for i in 0..grid_num {
            for j in 0..grid_num {
                let jitter_x = generator_context
                    .rng
                    .random_range(generator_setting.point_jitter_range.clone())
                    * grid_cell_size;
                let jitter_y = generator_context
                    .rng
                    .random_range(generator_setting.point_jitter_range.clone())
                    * grid_cell_size;

                let p = RegionPoint::from_xy(
                    (i as f32 * grid_cell_size + jitter_x) as f64,
                    (j as f32 * grid_cell_size + jitter_y) as f64,
                );
                points.push(p);
            }
        }
    }

    {
        let _span = info_span!("new terrain map").entered();

        let region = TerrainRegion::new(points);
        commands.insert_resource(region)
    }
}

fn generate_area_data(
    mut terrain_region: ResMut<TerrainRegion>,
    generator_setting: Res<TerrainRegionGeneratorSetting>,
    mut generator_context: ResMut<TerrainRegionGeneratorContext>,
    terrain_setting: Res<TerrainSetting>,
) {
    let mut parent_sites = vec![];
    let mut declines = vec![];
    let mut area_id = 0;

    let grid_num = generator_setting.get_grid_num(terrain_setting.get_terrain_size());

    // 每个Area生成一个随机点作为Area的中心
    for area_setting in generator_setting.rand_area_setting.iter() {
        let num = generator_context
            .rng
            .random_range(area_setting.rand_area_num.clone());
        for _ in 0..num {
            loop {
                let rng_range = area_setting.range_area_position_range.clone();
                let x = generator_context.rng.random_range(
                    rng_range.start().x * grid_num as f32..rng_range.end().x * grid_num as f32,
                ) as usize;
                let y = generator_context.rng.random_range(
                    rng_range.start().y * grid_num as f32..rng_range.end().y * grid_num as f32,
                ) as usize;
                // TODO 统一列主序还是行主序
                let index = x * grid_num + y;
                if parent_sites.contains(&index) {
                    continue;
                }

                terrain_region.sites_info[index].area_id = area_id;
                terrain_region.sites_info[index].biome_type = area_setting.biome_type;
                area_id += 1;

                terrain_region.sites_info[index].area_weight = 1.0;
                terrain_region.sites_info[index].pos = UVec2::new(x as u32, y as u32);
                parent_sites.push(index);

                terrain_region.area_random_points.push(index);

                let radius_rng_range = area_setting.rand_area_radius.clone();
                let r = generator_context.rng.random_range(radius_rng_range);
                if r == 0 {
                    declines.push(0.0);
                } else {
                    declines.push(1.0 / r as f32);
                }

                break;
            }
        }
    }

    // 展开每个Area的中心，向周围展开。
    // TODO: 没有解决两个Area的问题。或者这不是一个问题。
    // let mut count = 0;
    while !parent_sites.is_empty() {
        let mut children = vec![];
        for parent_index in parent_sites.iter() {
            let parent_site_info = terrain_region.sites_info[*parent_index];
            if parent_site_info.area_weight < 0.0 {
                continue;
            }

            let mut decline = declines[parent_site_info.area_id];

            let mut adjust = if decline > 0.0 {
                generator_context.rng.random_range(
                    -decline * 1.0..(decline * 0.5), // (-0.1 + 0.006 * count as f64).min(-0.02)..(0.1 - 0.006 * count as f64).max(0.01),
                )
            } else {
                0.0
            };

            if generator_setting
                .area_range
                .contains(&(parent_site_info.pos.x as f32 / grid_num as f32))
                && generator_setting
                    .area_range
                    .contains(&(parent_site_info.pos.y as f32 / grid_num as f32))
            {
                decline = 0.0;
                adjust = 0.0;
            }

            for neighbor_index in terrain_region.diagram.neighbors[*parent_index].clone() {
                let point = terrain_region.diagram.sites[neighbor_index];

                let neighbor_site_info = &mut terrain_region.sites_info[neighbor_index];
                if neighbor_site_info.area_weight != 0.0 {
                    continue;
                }

                // 并非均匀下降，而是有一定的随机调整，如此 Area 就不是都一样了
                neighbor_site_info.area_weight = parent_site_info.area_weight - decline + adjust;
                neighbor_site_info.area_id = parent_site_info.area_id;
                neighbor_site_info.biome_type = parent_site_info.biome_type;
                neighbor_site_info.pos =
                    (point.0.as_vec2() / generator_setting.grid_cell_size).as_uvec2();
                children.push(neighbor_index);
            }
        }
        info!("children num: {}", children.len());
        parent_sites = children;
        // count += 1;
    }

    info!("generate area over");
}

fn generate_biome_image(
    map: Res<TerrainRegion>,
    generator_setting: Res<TerrainRegionGeneratorSetting>,
    terrain_setting: Res<TerrainSetting>,
    mut terrain_region_info: ResMut<TerrainRegionInfo>,
    mut images: ResMut<Assets<Image>>,
) {
    let grid_num = generator_setting.get_grid_num(terrain_setting.get_terrain_size());

    let mut image = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::new(
        grid_num as u32 * generator_setting.grid_cell_size as u32,
        grid_num as u32 * generator_setting.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let biome_type = map.sites_info[i].biome_type as u8;
        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 && points[0] != *points.last().expect("") {
            draw_polygon_mut(&mut image, points.as_slice(), image::Luma([biome_type]));
            // draw_text(image, color, x, y, scale, font, text)
        }
    }

    let image = Image::from_dynamic(
        image::DynamicImage::ImageLuma8(image),
        false,
        RenderAssetUsages::RENDER_WORLD,
    );
    terrain_region_info.biome_image = images.add(image);
}

fn draw_area_image(
    map: Res<TerrainRegion>,
    generator_setting: Res<TerrainRegionGeneratorSetting>,
    terrain_setting: Res<TerrainSetting>,
) {
    let grid_num = generator_setting.get_grid_num(terrain_setting.get_terrain_size());

    let mut image = image::ImageBuffer::new(
        grid_num as u32 * generator_setting.grid_cell_size as u32,
        grid_num as u32 * generator_setting.grid_cell_size as u32,
    );

    for (i, cell) in map.diagram.cells.iter().enumerate() {
        let color = map.sites_info[i].biome_type.get_image_color();
        let color = image::Rgba([color[0], color[1], color[2], 255u8]);

        let points = cell
            .points()
            .iter()
            .map(|p| imageproc::point::Point::new((p.x()) as i32, (p.y()) as i32))
            .collect::<Vec<_>>();
        if points.len() > 2 && points[0] != *points.last().expect("") {
            draw_polygon_mut(&mut image, points.as_slice(), color);
            // draw_text(image, color, x, y, scale, font, text)
        }

        let neighbors = map.diagram.neighbors[i].clone();
        for neighbor in neighbors {
            if let Some([p0, p1]) = shared_edge(cell, &map.diagram.cells[neighbor]) {
                if map.sites_info[neighbor].area_id != map.sites_info[i].area_id {
                    draw_line_segment_mut(
                        &mut image,
                        (p0.x as f32, p0.y as f32),
                        (p1.x as f32, p1.y as f32),
                        image::Rgba([255u8, 255, 255, 255]),
                    );
                }
            }
        }
    }

    match image.save(generator_setting.image_save_path.join("terrain_area.png")) {
        Ok(_) => (),
        Err(e) => {
            error!("save terrain area image error: {}", e);
        }
    }
}

fn draw_delaunay_triangle_image(
    map: Res<TerrainRegion>,
    generator_setting: Res<TerrainRegionGeneratorSetting>,
    terrain_setting: Res<TerrainSetting>,
) {
    let grid_num = generator_setting.get_grid_num(terrain_setting.get_terrain_size());

    let mut image: ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(
        grid_num as u32 * generator_setting.grid_cell_size as u32,
        grid_num as u32 * generator_setting.grid_cell_size as u32,
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

    match image.save(
        generator_setting
            .image_save_path
            .join("terrain_delaunay.png"),
    ) {
        Ok(_) => (),
        Err(e) => {
            error!("save terrain delaunay image error: {}", e);
        }
    }
}

fn to_generate_terrain_mesh(mut state: ResMut<NextState<TerrainState>>) {
    state.set(TerrainState::GenerateTerrainMesh);
    info!("to_generate_terrain_mesh");
}
