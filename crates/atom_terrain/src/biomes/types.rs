use serde::{Deserialize, Serialize};

/**
 * 湖泊，和河流，仅作为特定地形的一种补充存在。
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    /**
     * 海洋
     */
    Ocean = 0,
    /**
     * 森林
     */
    Forest = 1,
    /**
     * 沙漠
     */
    Desert = 2,
    /**
     * 平原
     */
    Plains = 3,
    /**
     * 山地
     */
    Mountains = 4,
    /**
     * 沼泽
     */
    Swamp = 5,
}

impl BiomeType {
    pub(crate) fn get_image_color(&self) -> [u8; 3] {
        match self {
            BiomeType::Forest => [34, 139, 34],      // Forest Green
            BiomeType::Desert => [210, 180, 140],    // Tan
            BiomeType::Plains => [124, 252, 0],      // Lawn Green
            BiomeType::Mountains => [139, 137, 137], // Light Gray
            BiomeType::Swamp => [47, 79, 79],        // Dark Slate Gray
            BiomeType::Ocean => [70, 130, 180],      // Steel Blue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_colors_unique() {
        let biomes = [
            BiomeType::Ocean,
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Plains,
            BiomeType::Mountains,
            BiomeType::Swamp,
        ];
        let colors: Vec<_> = biomes.iter().map(|b| b.get_image_color()).collect();
        // All colors should be unique
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "duplicate color for {:?} and {:?}",
                    biomes[i], biomes[j]
                );
            }
        }
    }

    #[test]
    fn test_biome_forest_color() {
        assert_eq!(BiomeType::Forest.get_image_color(), [34, 139, 34]);
    }

    #[test]
    fn test_biome_ocean_color() {
        assert_eq!(BiomeType::Ocean.get_image_color(), [70, 130, 180]);
    }
}
