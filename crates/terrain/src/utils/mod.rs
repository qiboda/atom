use bevy::math::{
    bounding::{Aabb3d, BoundingVolume},
    Vec3A,
};
use terrain_core::chunk::coords::TerrainChunkCoord;

pub struct OctreeUtil;

impl OctreeUtil {
    pub fn get_subnode_aabb(parent_aabb: Aabb3d, subnode_index: u8) -> Aabb3d {
        let mut child_aabb = parent_aabb;
        let parent_half_size = parent_aabb.half_size();
        if subnode_index & 0b001 == 0b001 {
            child_aabb.min.x += parent_half_size.x;
        } else {
            child_aabb.max.x -= parent_half_size.x;
        }

        if subnode_index & 0b010 == 0b010 {
            child_aabb.min.y += parent_half_size.y;
        } else {
            child_aabb.max.y -= parent_half_size.y;
        }

        if subnode_index & 0b100 == 0b100 {
            child_aabb.min.z += parent_half_size.z;
        } else {
            child_aabb.max.z -= parent_half_size.z;
        }

        child_aabb
    }

    pub fn get_parent_node_aabb(current_aabb: Aabb3d, current_node_index: u8) -> Aabb3d {
        let mut parent_aabb = current_aabb;
        let parent_half_size = current_aabb.half_size() * 2.0;

        // x轴右边
        if current_node_index & 0b001 == 0b001 {
            parent_aabb.min.x -= parent_half_size.x;
        } else {
            parent_aabb.max.x += parent_half_size.x;
        }

        if current_node_index & 0b010 == 0b010 {
            parent_aabb.min.y -= parent_half_size.y;
        } else {
            parent_aabb.max.y += parent_half_size.y;
        }

        if current_node_index & 0b100 == 0b100 {
            parent_aabb.min.z -= parent_half_size.z;
        } else {
            parent_aabb.max.z += parent_half_size.z;
        }

        parent_aabb
    }
}

pub struct TerrainChunkUtils;

impl TerrainChunkUtils {
    pub fn get_coord_from_location(chunk_size: f32, location: Vec3A) -> TerrainChunkCoord {
        TerrainChunkCoord::from(location / chunk_size)
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    #[test]
    fn test_subnode_aabb() {
        use super::*;
        let parent_aabb = Aabb3d::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        let subnode_aabb = OctreeUtil::get_subnode_aabb(parent_aabb, 0);
        assert_eq!(subnode_aabb.min, Vec3A::new(-10.0, -10.0, -10.0));
        assert_eq!(subnode_aabb.max, Vec3A::new(0.0, 0.0, 0.0));

        let subnode_aabb = OctreeUtil::get_subnode_aabb(parent_aabb, 2);
        assert_eq!(subnode_aabb.min, Vec3A::new(-10.0, 0.0, -10.0));
        assert_eq!(subnode_aabb.max, Vec3A::new(0.0, 10.0, 0.0));
    }
}
