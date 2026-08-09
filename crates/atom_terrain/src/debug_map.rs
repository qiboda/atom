//! 调试用：2K 温度图、高度图、地表类型图导出。
//! 全地图 4096m × 4096m，2048×2048 分辨率（2m/px）。

use bevy::prelude::*;

use crate::biome::{self, SurfaceType};

/// 生成全地图温度图、高度图和地表图的 startup system。
pub fn generate_debug_maps_system() {
    const MAP_SIZE: f32 = 4096.0;
    const RES: u32 = 2048;
    let scale = MAP_SIZE / RES as f32;
    let half = MAP_SIZE / 2.0;

    info!("Generating 2K debug maps ({RES}×{RES}, {MAP_SIZE}m range)...");

    // Prescan at 512×512 for range estimation
    let pre_res: u32 = 512;
    let pre_scale = MAP_SIZE / pre_res as f32;
    let mut h_min = f32::MAX;
    let mut h_max = f32::MIN;
    let mut t_min = f32::MAX;
    let mut t_max = f32::MIN;

    for py in 0..pre_res {
        let z = -half + py as f32 * pre_scale;
        for px in 0..pre_res {
            let x = -half + px as f32 * pre_scale;
            let h = crate::noise::height_at(x, z);
            h_min = h_min.min(h);
            h_max = h_max.max(h);
            let cx = (x / 30.0).floor() as i32;
            let cz = (z / 30.0).floor() as i32;
            let t = crate::noise::cell_temperature(cx, cz, 42);
            t_min = t_min.min(t);
            t_max = t_max.max(t);
        }
    }

    let h_range = (h_max - h_min).max(0.01);
    let t_range = (t_max - t_min).max(0.01);
    info!("  height: [{h_min:.1}, {h_max:.1}]  temp: [{t_min:.2}, {t_max:.2}]");

    let mut height_pixels = Vec::with_capacity((RES * RES) as usize);
    let mut temp_pixels = Vec::with_capacity((RES * RES) as usize);
    let mut surface_pixels = Vec::with_capacity((RES * RES * 3) as usize);

    for py in 0..RES {
        let z = -half + py as f32 * scale;
        for px in 0..RES {
            let x = -half + px as f32 * scale;

            let h = crate::noise::height_at(x, z);
            let gray = ((h - h_min) / h_range * 255.0) as u8;
            height_pixels.push(gray);

            let cx = (x / 30.0).floor() as i32;
            let cz = (z / 30.0).floor() as i32;
            let t = crate::noise::cell_temperature(cx, cz, 42);
            let t_gray = ((t - t_min) / t_range * 255.0) as u8;
            temp_pixels.push(t_gray);

            let st = biome::surface_at(x, z, 42);
            let (r, g, b) = surface_color(st);
            surface_pixels.push(r);
            surface_pixels.push(g);
            surface_pixels.push(b);
        }
    }

    // Height map
    let header = format!("P5\n{RES} {RES}\n255\n");
    let mut pgm = header.into_bytes();
    pgm.extend_from_slice(&height_pixels);
    if let Err(e) = std::fs::write("heightmap.pgm", &pgm) {
        error!("  heightmap.pgm: {e}");
    }

    // Temperature map
    let header = format!("P5\n{RES} {RES}\n255\n");
    let mut pgm = header.into_bytes();
    pgm.extend_from_slice(&temp_pixels);
    match std::fs::write("tempmap.pgm", &pgm) {
        Ok(()) => info!("  tempmap.pgm ({RES}×{RES})"),
        Err(e) => error!("  tempmap.pgm: {e}"),
    }

    // Surface type map
    let header = format!("P6\n{RES} {RES}\n255\n");
    let mut ppm = header.into_bytes();
    ppm.extend_from_slice(&surface_pixels);
    match std::fs::write("surfacemap.ppm", &ppm) {
        Ok(()) => info!("  surfacemap.ppm ({RES}×{RES})"),
        Err(e) => error!("  surfacemap.ppm: {e}"),
    }

    info!("  done.");
}

fn surface_color(st: SurfaceType) -> (u8, u8, u8) {
    match st {
        SurfaceType::Snow => (247, 247, 255),
        SurfaceType::Tundra => (186, 186, 209),
        SurfaceType::Taiga => (153, 171, 120),
        SurfaceType::Forest => (69, 140, 51),
        SurfaceType::Grassland => (135, 179, 64),
        SurfaceType::Desert => (209, 196, 140),
        SurfaceType::Rock => (115, 115, 128),
        SurfaceType::Swamp => (64, 89, 51),
        SurfaceType::Ocean => (46, 64, 140),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_color_maps_all_types() {
        assert_eq!(surface_color(SurfaceType::Snow), (247, 247, 255));
        assert_eq!(surface_color(SurfaceType::Tundra), (186, 186, 209));
        assert_eq!(surface_color(SurfaceType::Taiga), (153, 171, 120));
        assert_eq!(surface_color(SurfaceType::Forest), (69, 140, 51));
        assert_eq!(surface_color(SurfaceType::Grassland), (135, 179, 64));
        assert_eq!(surface_color(SurfaceType::Desert), (209, 196, 140));
        assert_eq!(surface_color(SurfaceType::Rock), (115, 115, 128));
        assert_eq!(surface_color(SurfaceType::Swamp), (64, 89, 51));
        assert_eq!(surface_color(SurfaceType::Ocean), (46, 64, 140));
    }

    #[test]
    fn generate_debug_maps_writes_pgm_and_ppm_files() {
        generate_debug_maps_system();

        for f in ["heightmap.pgm", "tempmap.pgm", "surfacemap.ppm"] {
            let meta = std::fs::metadata(f).expect("debug map 文件应已生成");
            assert!(meta.len() > 0, "{f} 应为非空文件");
            std::fs::remove_file(f).expect("清理 debug map 文件");
        }
    }
}
