use bevy::{prelude::*, utils::HashMap};

use crate::{
    chunk_mgr::chunk::{
        bundle::TerrainChunkBundle,
        chunk_lod::{TerrainChunkAabb, TerrainChunkLod},
        state::{TerrainChunkAddress, TerrainChunkState},
    },
    isosurface::comp::TerrainChunkCreateMainMeshEvent,
    lod::lod_octree::{LodOctreeMap, LodOctreeNode},
    setting::TerrainSetting,
};

use super::{
    chunk::bundle::TerrainChunk,
    chunk_loader::{TerrainChunkLoadEvent, TerrainChunkReloadEvent, TerrainChunkUnLoadEvent},
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

pub fn trigger_chunk_reload_event(
    event_trigger: Trigger<TerrainChunkReloadEvent>,
    terrain_chunk_mapper: Res<TerrainChunkMapper>,
    mut query: Query<(Entity, &mut TerrainChunkState, &mut TerrainChunkLod), With<TerrainChunk>>,
    mut commands: Commands,
) {
    for node_address in event_trigger.event().node_addresses.iter() {
        let terrain_chunk_address: TerrainChunkAddress = node_address.into();
        let Some(entity) = terrain_chunk_mapper.data.get(&terrain_chunk_address) else {
            return;
        };
        if let Ok((chunk_entity, mut state, chunk_lod)) = query.get_mut(*entity) {
            *state = TerrainChunkState::CreateMainMesh;
            commands.trigger(TerrainChunkCreateMainMeshEvent {
                entity: chunk_entity,
                lod: chunk_lod.get_lod(),
            });
        }
    }
}

pub fn trigger_chunk_load_event(
    event_trigger: Trigger<TerrainChunkLoadEvent>,
    mut commands: Commands,
    mut terrain_chunk_mapper: ResMut<TerrainChunkMapper>,
    query: Query<&LodOctreeNode>,
    lod_octree_map: Res<LodOctreeMap>,
    terrain_settings: Res<TerrainSetting>,
) {
    for node_address in event_trigger.event().node_addresses.iter() {
        let Some(node_entity) = lod_octree_map.get_node_entity(*node_address) else {
            continue;
        };

        let Ok(lod_octree_node) = query.get(*node_entity) else {
            return;
        };

        let terrain_chunk_address = lod_octree_node.address.into();
        if terrain_chunk_mapper
            .data
            .contains_key(&terrain_chunk_address)
        {
            panic!("repeat load chunk: {:?}", terrain_chunk_address);
        }

        let mut bundle = TerrainChunkBundle::new(TerrainChunkState::CreateMainMesh);
        bundle.terrain_chunk_address = terrain_chunk_address;
        let lod_octree_depth = lod_octree_node.address.get_depth();
        let chunk_lod = terrain_settings.lod_setting.get_lod_octree_depth() - lod_octree_depth;
        bundle.terrain_chunk_lod = TerrainChunkLod::new(chunk_lod);
        bundle.terrain_chunk_aabb = TerrainChunkAabb(lod_octree_node.aabb);

        info!(
            "spawn_terrain_chunks: {:?}, lod: {}",
            bundle.terrain_chunk_address, chunk_lod
        );

        let chunk_entity = commands.spawn(bundle).id();
        let value = terrain_chunk_mapper
            .data
            .insert(terrain_chunk_address, chunk_entity);
        assert!(value.is_none());

        commands.trigger(TerrainChunkCreateMainMeshEvent {
            entity: chunk_entity,
            lod: chunk_lod,
        });
    }
}

pub fn trigger_chunk_unload_event(
    event_trigger: Trigger<TerrainChunkUnLoadEvent>,
    mut terrain_chunk_mapper: ResMut<TerrainChunkMapper>,
    mut commands: Commands,
) {
    for node_address in event_trigger.event().node_addresses.iter() {
        let terrain_chunk_address = node_address.into();
        if let Some(chunk_entity) = terrain_chunk_mapper.get_chunk_entity(terrain_chunk_address) {
            commands.get_entity(*chunk_entity).map(|x| {
                x.despawn_recursive();
                // info!(
                //     "trigger_chunk_unload_event despawn: {:?}",
                //     terrain_chunk_address
                // );
                terrain_chunk_mapper.data.remove(&terrain_chunk_address)
            });
        }
    }
}
