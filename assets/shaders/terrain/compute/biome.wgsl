#define_import_path terrain::biome

#import noise::fbm::open_simplex_2d_fbm_with_seed
#import math::map::map

// 地形noise，生态noise
const TerrainType_Ocean:u32 = 0;
const TerrainType_Lake:u32 = 1;
const TerrainType_Beach:u32 = 2;

const TerrainType_PlainSwamp:u32 = 3;
const TerrainType_PlainDesert:u32 = 4;
const TerrainType_PlainRainForest:u32 = 5;
const TerrainType_PlainForest:u32 = 6;
const TerrainType_PlainGrassLand:u32 = 7;
const TerrainType_PlainSnow:u32 = 8;
const TerrainType_PlainIce:u32 = 9;

const TerrainType_HillsSwamp:u32 = 10;
const TerrainType_HillsDesert:u32 = 11;
const TerrainType_HillsRainForest:u32 = 12;
const TerrainType_HillsForest:u32 = 13;
const TerrainType_HillsGrassLand:u32 = 14;
const TerrainType_HillsSnow:u32 = 15;
const TerrainType_HillsIce:u32 = 16;

const TerrainType_MountainCommon:u32 = 17;
const TerrainType_MountainSnow:u32 = 18;
const TerrainType_MountainVolcano:u32 = 19;

const TerrainType_Underground:u32 = 20;

const TerrainType_MAX:u32 = 21;

const terrain_height: f32 = 1.0;

// -40
const ocean_low : f32 = - terrain_height * 0.4;
// -1
const beach_low : f32 = - terrain_height * 0.01;
// 1
const plain_low : f32 = terrain_height * 0.01;
// 5
const hills_low : f32 = terrain_height * 0.05;
// 30
const mountain_low : f32 = terrain_height * 0.5;

fn get_ocean_noise(location: vec2f) -> f32 {
    let basic_noise = open_simplex_2d_fbm_with_seed(location, 323u, 3u, 0.000025, 2.0, 2.0);
    let value = map(basic_noise, -1.0, 1.0, ocean_low, beach_low);
    return value;
}

fn get_lake_noise(location: vec2f) -> f32 {
    let basic_noise = open_simplex_2d_fbm_with_seed(location, 323u, 3u, 0.001, 2.0, 2.0);
    let value = map(basic_noise, -1.0, 1.0, ocean_low, beach_low);
    return value;
}

fn get_beach_noise(location: vec2f) -> f32 {
    let basic_noise = open_simplex_2d_fbm_with_seed(location, 323u, 3u, 0.001, 2.0, 2.0);
    let value = map(basic_noise, -1.0, 1.0, beach_low, plain_low);
    return value;
}

fn get_plain_noise(location: vec2f) -> f32 {
    let basic_noise = open_simplex_2d_fbm_with_seed(location, 323u, 3u, 0.001, 2.0, 2.0);
    let value = map(basic_noise, -1.0, 1.0, plain_low, hills_low);
    return value;
}

fn get_hills_noise(location: vec2f) -> f32 {
    let basic_noise = open_simplex_2d_fbm_with_seed(location, 323u, 1u, 0.001, 1.0, 1.0);
    let value = map(basic_noise, -1.0, 1.0, hills_low, mountain_low);
    return value;
}

fn get_mountain_noise(location: vec2f) -> f32 {
    let basic_noise = open_simplex_2d_fbm_with_seed(location, 323u, 3u, 0.001, 2.0, 2.0);
    let value = map(basic_noise, -1.0, 1.0, mountain_low, terrain_height);
    return value;
}


fn get_terrain_noise(location: vec2f, biome_type: u32) -> f32 {
    switch biome_type {
        case TerrainType_Ocean {
            return get_ocean_noise(location);
        }
        case TerrainType_Lake {
            return get_lake_noise(location);
        }
        case TerrainType_Beach {
            return get_beach_noise(location);
        }
        case TerrainType_PlainSwamp {
            return get_plain_noise(location);
        }
        case TerrainType_PlainDesert {
            return get_plain_noise(location);
        }
        case TerrainType_PlainRainForest {
            return get_plain_noise(location);
        }
        case TerrainType_PlainForest {
            return get_plain_noise(location);
        }
        case TerrainType_PlainGrassLand {
            return get_plain_noise(location);
        }
        case TerrainType_PlainSnow {
            return get_plain_noise(location);
        }
        case TerrainType_PlainIce {
            return get_plain_noise(location);
        }

        case TerrainType_HillsSwamp {
            return get_hills_noise(location);
        }
        case TerrainType_HillsDesert {
            return get_hills_noise(location);
        }
        case TerrainType_HillsRainForest {
            return get_hills_noise(location);
        }
        case TerrainType_HillsForest {
            return get_hills_noise(location);
        }
        case TerrainType_HillsGrassLand {
            return get_hills_noise(location);
        }
        case TerrainType_HillsSnow {
            return get_hills_noise(location);
        }
        case TerrainType_HillsIce {
            return get_hills_noise(location);
        }

        case TerrainType_MountainCommon {
            return get_mountain_noise(location);
        }
        case TerrainType_MountainSnow {
            return get_mountain_noise(location);
        }
        case TerrainType_MountainVolcano {
            return get_mountain_noise(location);
        }
        default: {
            return 0.0;
        }
    }
    return 0.0;
}
