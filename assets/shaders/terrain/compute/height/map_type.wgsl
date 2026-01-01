#define_import_path terrain::map::map_type

struct TerrainMapInfo {
    terrain_size: f32,
    map_size: f32,
    pixel_num_per_kernel: u32,
    stride: u32,
}
