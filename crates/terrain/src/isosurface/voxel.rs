use bevy::reflect::Reflect;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum VoxelMaterialType {
    Air,
    Block,
}

impl From<u32> for VoxelMaterialType {
    fn from(value: u32) -> Self {
        match value {
            0 => VoxelMaterialType::Air,
            1 => VoxelMaterialType::Block,
            _ => panic!("Invalid VoxelMaterialType value: {}", value),
            // _ => VoxelMaterialType::Air,
        }
    }
}
