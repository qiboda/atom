use core::panic;

// 和shader保持一致
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::FromRepr)]
pub enum MapFlatTerrainType {
    Ocean = 0,
    Lake = 1,
    Beach = 2,

    PlainSwamp = 3,
    PlainDesert = 4,
    PlainRainForest = 5,
    PlainForest = 6,
    PlainGrassLand = 7,
    PlainSnow = 8,
    PlainIce = 9,

    HillsSwamp = 10,
    HillsDesert = 11,
    HillsRainForest = 12,
    HillsForest = 13,
    HillsGrassLand = 14,
    HillsSnow = 15,
    HillsIce = 16,

    MountainCommon = 17,
    MountainSnow = 18,
    MountainVolcano = 19,

    Underground = 20,
}

impl MapFlatTerrainType {
    pub const MAX: usize = 21;
    pub const INVALID: u32 = 255;
}

impl MapFlatTerrainType {
    pub fn is_surface_type(&self) -> bool {
        match self {
            MapFlatTerrainType::Ocean => true,
            MapFlatTerrainType::Lake => true,
            MapFlatTerrainType::Beach => true,
            MapFlatTerrainType::PlainSwamp => true,
            MapFlatTerrainType::PlainDesert => true,
            MapFlatTerrainType::PlainRainForest => true,
            MapFlatTerrainType::PlainForest => true,
            MapFlatTerrainType::PlainGrassLand => true,
            MapFlatTerrainType::PlainSnow => true,
            MapFlatTerrainType::PlainIce => true,
            MapFlatTerrainType::HillsSwamp => true,
            MapFlatTerrainType::HillsDesert => true,
            MapFlatTerrainType::HillsRainForest => true,
            MapFlatTerrainType::HillsForest => true,
            MapFlatTerrainType::HillsGrassLand => true,
            MapFlatTerrainType::HillsSnow => true,
            MapFlatTerrainType::HillsIce => true,
            MapFlatTerrainType::MountainCommon => true,
            MapFlatTerrainType::MountainSnow => true,
            MapFlatTerrainType::MountainVolcano => true,
            MapFlatTerrainType::Underground => false,
        }
    }

    pub fn is_underground_type(&self) -> bool {
        match self {
            MapFlatTerrainType::Ocean => false,
            MapFlatTerrainType::Lake => false,
            MapFlatTerrainType::Beach => false,
            MapFlatTerrainType::PlainSwamp => false,
            MapFlatTerrainType::PlainDesert => false,
            MapFlatTerrainType::PlainRainForest => false,
            MapFlatTerrainType::PlainForest => false,
            MapFlatTerrainType::PlainGrassLand => false,
            MapFlatTerrainType::PlainSnow => false,
            MapFlatTerrainType::PlainIce => false,
            MapFlatTerrainType::HillsSwamp => false,
            MapFlatTerrainType::HillsDesert => false,
            MapFlatTerrainType::HillsRainForest => false,
            MapFlatTerrainType::HillsForest => false,
            MapFlatTerrainType::HillsGrassLand => false,
            MapFlatTerrainType::HillsSnow => false,
            MapFlatTerrainType::HillsIce => false,
            MapFlatTerrainType::MountainCommon => false,
            MapFlatTerrainType::MountainSnow => false,
            MapFlatTerrainType::MountainVolcano => false,
            MapFlatTerrainType::Underground => true,
        }
    }
}

impl From<MapTerrainType> for MapFlatTerrainType {
    fn from(value: MapTerrainType) -> Self {
        match value {
            MapTerrainType::Ocean => MapFlatTerrainType::Ocean,
            MapTerrainType::Lake => MapFlatTerrainType::Lake,
            MapTerrainType::Beach => MapFlatTerrainType::Beach,
            MapTerrainType::Plain(landform) => match landform {
                MapPlainLandform::Swamp => MapFlatTerrainType::PlainSwamp,
                MapPlainLandform::Desert => MapFlatTerrainType::PlainDesert,
                MapPlainLandform::RainForest => MapFlatTerrainType::PlainRainForest,
                MapPlainLandform::Forest => MapFlatTerrainType::PlainForest,
                MapPlainLandform::GrassLand => MapFlatTerrainType::PlainGrassLand,
                MapPlainLandform::Snow => MapFlatTerrainType::PlainSnow,
                MapPlainLandform::Ice => MapFlatTerrainType::PlainIce,
            },
            MapTerrainType::Hills(landform) => match landform {
                MapHillsLandform::Swamp => MapFlatTerrainType::HillsSwamp,
                MapHillsLandform::Desert => MapFlatTerrainType::HillsDesert,
                MapHillsLandform::RainForest => MapFlatTerrainType::HillsRainForest,
                MapHillsLandform::Forest => MapFlatTerrainType::HillsForest,
                MapHillsLandform::GrassLand => MapFlatTerrainType::HillsGrassLand,
                MapHillsLandform::Snow => MapFlatTerrainType::HillsSnow,
                MapHillsLandform::Ice => MapFlatTerrainType::HillsIce,
            },
            MapTerrainType::Mountain(landform) => match landform {
                MapMountainLandform::Common => MapFlatTerrainType::MountainCommon,
                MapMountainLandform::Snow => MapFlatTerrainType::MountainSnow,
                MapMountainLandform::Volcano => MapFlatTerrainType::MountainVolcano,
            },
            MapTerrainType::Underground(underground) => match underground {
                MapUndergroundLandform::Common => MapFlatTerrainType::Underground,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapTerrainType {
    Ocean,
    Lake,
    Beach,
    Plain(MapPlainLandform),
    Hills(MapHillsLandform),
    Mountain(MapMountainLandform),
    Underground(MapUndergroundLandform),
}

impl From<MapFlatTerrainType> for MapTerrainType {
    fn from(flat_type: MapFlatTerrainType) -> MapTerrainType {
        match flat_type {
            MapFlatTerrainType::Ocean => MapTerrainType::Ocean,
            MapFlatTerrainType::Lake => MapTerrainType::Lake,
            MapFlatTerrainType::Beach => MapTerrainType::Beach,
            MapFlatTerrainType::PlainSwamp => MapTerrainType::Plain(MapPlainLandform::Swamp),
            MapFlatTerrainType::PlainDesert => MapTerrainType::Plain(MapPlainLandform::Desert),
            MapFlatTerrainType::PlainRainForest => {
                MapTerrainType::Plain(MapPlainLandform::RainForest)
            }
            MapFlatTerrainType::PlainForest => MapTerrainType::Plain(MapPlainLandform::Forest),
            MapFlatTerrainType::PlainGrassLand => {
                MapTerrainType::Plain(MapPlainLandform::GrassLand)
            }
            MapFlatTerrainType::PlainSnow => MapTerrainType::Plain(MapPlainLandform::Snow),
            MapFlatTerrainType::PlainIce => MapTerrainType::Plain(MapPlainLandform::Ice),
            MapFlatTerrainType::HillsSwamp => MapTerrainType::Hills(MapHillsLandform::Swamp),
            MapFlatTerrainType::HillsDesert => MapTerrainType::Hills(MapHillsLandform::Desert),
            MapFlatTerrainType::HillsRainForest => {
                MapTerrainType::Hills(MapHillsLandform::RainForest)
            }
            MapFlatTerrainType::HillsForest => MapTerrainType::Hills(MapHillsLandform::Forest),
            MapFlatTerrainType::HillsGrassLand => {
                MapTerrainType::Hills(MapHillsLandform::GrassLand)
            }
            MapFlatTerrainType::HillsSnow => MapTerrainType::Hills(MapHillsLandform::Snow),
            MapFlatTerrainType::HillsIce => MapTerrainType::Hills(MapHillsLandform::Ice),

            MapFlatTerrainType::MountainCommon => {
                MapTerrainType::Mountain(MapMountainLandform::Common)
            }
            MapFlatTerrainType::MountainSnow => MapTerrainType::Mountain(MapMountainLandform::Snow),
            MapFlatTerrainType::MountainVolcano => {
                MapTerrainType::Mountain(MapMountainLandform::Volcano)
            }
            MapFlatTerrainType::Underground => {
                MapTerrainType::Underground(MapUndergroundLandform::Common)
            }
        }
    }
}

impl MapTerrainType {
    pub fn terrain_type_eq(&self, other: &MapTerrainType) -> bool {
        matches!(
            (self, other),
            (MapTerrainType::Ocean, MapTerrainType::Ocean)
                | (MapTerrainType::Lake, MapTerrainType::Lake)
                | (MapTerrainType::Beach, MapTerrainType::Beach)
                | (MapTerrainType::Plain(_), MapTerrainType::Plain(_))
                | (MapTerrainType::Hills(_), MapTerrainType::Hills(_))
                | (MapTerrainType::Mountain(_), MapTerrainType::Mountain(_))
        )
    }

    pub fn topography_eq(&self, other: &MapTerrainType) -> bool {
        self == other
    }

    pub fn get_all_color() -> Vec<[u8; 4]> {
        vec![
            MapTerrainType::Ocean.get_color(),
            MapTerrainType::Lake.get_color(),
            MapTerrainType::Beach.get_color(),
            MapTerrainType::Plain(MapPlainLandform::Swamp).get_color(),
            MapTerrainType::Plain(MapPlainLandform::Desert).get_color(),
            MapTerrainType::Plain(MapPlainLandform::RainForest).get_color(),
            MapTerrainType::Plain(MapPlainLandform::Forest).get_color(),
            MapTerrainType::Plain(MapPlainLandform::GrassLand).get_color(),
            MapTerrainType::Plain(MapPlainLandform::Snow).get_color(),
            MapTerrainType::Plain(MapPlainLandform::Ice).get_color(),
            MapTerrainType::Hills(MapHillsLandform::Swamp).get_color(),
            MapTerrainType::Hills(MapHillsLandform::Desert).get_color(),
            MapTerrainType::Hills(MapHillsLandform::RainForest).get_color(),
            MapTerrainType::Hills(MapHillsLandform::Forest).get_color(),
            MapTerrainType::Hills(MapHillsLandform::GrassLand).get_color(),
            MapTerrainType::Hills(MapHillsLandform::Snow).get_color(),
            MapTerrainType::Hills(MapHillsLandform::Ice).get_color(),
            MapTerrainType::Mountain(MapMountainLandform::Common).get_color(),
            MapTerrainType::Mountain(MapMountainLandform::Snow).get_color(),
            MapTerrainType::Mountain(MapMountainLandform::Volcano).get_color(),
        ]
    }

    pub fn get_color(&self) -> [u8; 4] {
        let color: [u8; 4] = match self {
            MapTerrainType::Ocean => [0, 0, 255, 255],
            MapTerrainType::Lake => [0, 255, 255, 255],
            MapTerrainType::Beach => [200, 0, 200, 255],
            MapTerrainType::Plain(landform) => match landform {
                MapPlainLandform::Swamp => [139, 0, 0, 255],
                MapPlainLandform::Desert => [205, 133, 0, 255],
                MapPlainLandform::RainForest => [144, 238, 144, 255],
                MapPlainLandform::Forest => [0, 139, 139, 255],
                MapPlainLandform::GrassLand => [0, 255, 127, 255],
                MapPlainLandform::Snow => [255, 250, 250, 255],
                MapPlainLandform::Ice => [220, 220, 220, 255],
            },
            MapTerrainType::Hills(landform) => match landform {
                MapHillsLandform::Swamp => [139, 0, 0, 255],
                MapHillsLandform::Desert => [205, 133, 0, 255],
                MapHillsLandform::RainForest => [144, 238, 144, 255],
                MapHillsLandform::Forest => [0, 139, 139, 255],
                MapHillsLandform::GrassLand => [0, 255, 127, 255],
                MapHillsLandform::Snow => [255, 250, 250, 255],
                MapHillsLandform::Ice => [220, 220, 220, 255],
            },
            MapTerrainType::Mountain(landform) => match landform {
                MapMountainLandform::Common => [250, 250, 0, 255],
                MapMountainLandform::Snow => [255, 250, 250, 255],
                MapMountainLandform::Volcano => [255, 0, 0, 255],
            },
            MapTerrainType::Underground(_) => [0, 0, 0, 255],
        };
        color
    }
}

/// 地貌
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapPlainLandform {
    Swamp,

    Desert,

    RainForest,
    Forest,
    GrassLand,

    Snow,
    Ice,
}

impl MapPlainLandform {
    pub fn determine_landform(temperature: f64, humidity: f64) -> MapPlainLandform {
        match temperature {
            -40.0..-20.0 => MapPlainLandform::Ice,
            -20.0..-0.0 => MapPlainLandform::Snow,
            0.0..30.0 => {
                // 应该增加一个随机概率
                if humidity < 0.3 {
                    MapPlainLandform::Forest
                } else if humidity < 0.7 {
                    MapPlainLandform::GrassLand
                } else {
                    MapPlainLandform::Swamp
                }
            }
            30.0..40.0 => {
                if humidity < 0.3 {
                    MapPlainLandform::Desert
                } else {
                    MapPlainLandform::RainForest
                }
            }
            _ => {
                panic!("temperature out of range");
            }
        }
    }
}

/// 地貌
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapHillsLandform {
    Swamp,

    Desert,

    RainForest,
    Forest,
    GrassLand,

    Snow,
    Ice,
}

impl MapHillsLandform {
    pub fn determine_landform(temperature: f64, humidity: f64) -> MapHillsLandform {
        match temperature {
            -40.0..-20.0 => MapHillsLandform::Ice,
            -20.0..-0.0 => MapHillsLandform::Snow,
            0.0..30.0 => {
                // 应该增加一个随机概率
                if humidity < 0.3 {
                    MapHillsLandform::Forest
                } else if humidity < 0.6 {
                    MapHillsLandform::GrassLand
                } else {
                    MapHillsLandform::Swamp
                }
            }
            30.0..40.0 => {
                if humidity < 0.3 {
                    MapHillsLandform::Desert
                } else {
                    MapHillsLandform::RainForest
                }
            }
            _ => {
                panic!("temperature out of range");
            }
        }
    }
}

/// 地貌
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapMountainLandform {
    Common,
    Snow,
    Volcano,
}

impl MapMountainLandform {
    pub fn determine_landform(temperature: f64, _humidity: f64) -> MapMountainLandform {
        match temperature {
            -40.0..0.0 => MapMountainLandform::Snow,
            0.0..30.0 => MapMountainLandform::Common,
            30.0..40.0 => MapMountainLandform::Volcano,
            _ => {
                panic!("temperature out of range");
            }
        }
    }
}

/// 地貌
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapUndergroundLandform {
    Common,
}

impl MapUndergroundLandform {
    pub fn determine_landform(_temperature: f64, _humidity: f64) -> MapUndergroundLandform {
        MapUndergroundLandform::Common
    }
}
