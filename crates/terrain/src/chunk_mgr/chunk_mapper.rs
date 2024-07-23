use bevy::{prelude::*, utils::HashMap};

use crate::{
    chunk_mgr::chunk::{
        bundle::TerrainChunkBundle,
        chunk_lod::{TerrainChunkAabb, TerrainChunkLod},
        state::{TerrainChunkAddress, TerrainChunkState},
    },
    isosurface::comp::TerrainChunkCreateMainMeshEvent,
    lod::lod_octree::{LodOctreeNode, LodOctreeNodeType},
    setting::TerrainSetting,
};

#[derive(Debug, Resource, Default, Reflect)]
pub struct TerrainChunkMapper {
    /// entity is TerrainChunk
    pub data: HashMap<TerrainChunkAddress, Entity>,
}

impl TerrainChunkMapper {
    pub fn get_chunk_entity(&self, terrain_chunk_address: TerrainChunkAddress) -> Option<&Entity> {
        self.data.get(&terrain_chunk_address)
    }

    pub fn new() -> TerrainChunkMapper {
        Self::default()
    }
}

pub fn trigger_lod_octree_remove(
    trigger: Trigger<OnRemove, LodOctreeNode>,
    query: Query<&LodOctreeNode>,
    mut commands: Commands,
    mut terrain_chunk_mapper: ResMut<TerrainChunkMapper>,
) {
    let entity = trigger.entity();
    if let Ok(lod_octree_node) = query.get(entity) {
        if lod_octree_node.node_type == LodOctreeNodeType::Internal {
            return;
        }
        let address: TerrainChunkAddress = lod_octree_node.address.into();
        if let Some(chunk_entity) = terrain_chunk_mapper.get_chunk_entity(address) {
            commands.get_entity(*chunk_entity).map(|x| {
                x.despawn_recursive();
                terrain_chunk_mapper.data.remove(&address)
            });
        }
    }
}

pub fn trigger_lod_octree_add(
    trigger: Trigger<OnAdd, LodOctreeNode>,
    query: Query<&LodOctreeNode>,
    mut commands: Commands,
    mut terrain_chunk_mapper: ResMut<TerrainChunkMapper>,
    mut event_writer: EventWriter<TerrainChunkCreateMainMeshEvent>,
    terrain_settings: Res<TerrainSetting>,
) {
    let entity = trigger.entity();
    warn!("trigger_lod_octree_add");
    if let Ok(lod_octree_node) = query.get(entity) {
        if lod_octree_node.node_type == LodOctreeNodeType::Internal {
            return;
        }
        let mut bundle = TerrainChunkBundle::new(TerrainChunkState::CreateMainMesh);
        let terrain_chunk_address = lod_octree_node.address.into();
        bundle.terrain_chunk_address = terrain_chunk_address;
        let lod_octree_depth = lod_octree_node.address.get_depth();
        let chunk_lod = terrain_settings.lod_setting.get_lod_octree_depth() - lod_octree_depth;
        bundle.terrain_chunk_lod = TerrainChunkLod::new(chunk_lod);
        bundle.terrain_chunk_aabb = TerrainChunkAabb(lod_octree_node.aabb);

        warn!(
            "spawn_terrain_chunks: {:?}, lod: {}",
            bundle.terrain_chunk_address, chunk_lod
        );
        let chunk_entity = commands.spawn(bundle).id();
        terrain_chunk_mapper
            .data
            .insert(terrain_chunk_address, chunk_entity);

        event_writer.send(TerrainChunkCreateMainMeshEvent {
            entity: chunk_entity,
            lod: chunk_lod,
        });
        info!(
            "update terrain chunk address: {:?}, lod: {}",
            terrain_chunk_address, chunk_lod
        );
    }
}
