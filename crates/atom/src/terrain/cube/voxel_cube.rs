struct VoxelCubeFaceData {
    pub vertex: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

enum VoxelCubeFace {
    Top(VoxelCubeFaceData),
    Bottom(VoxelCubeFaceData),
    Left(VoxelCubeFaceData),
    Right(VoxelCubeFaceData),
    Front(VoxelCubeFaceData),
    Back(VoxelCubeFaceData),
}

struct VoxelCubeData {
    faces: Vec<VoxelCubeFace>,
}
