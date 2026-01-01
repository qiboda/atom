use bevy::prelude::*;

#[derive(Debug, Component, Clone)]
#[relationship(relationship_target = TerrainChunkLogic)]
pub struct TerrainChunkVisual(Entity);

#[derive(Debug, Component, Clone)]
#[relationship_target(relationship = TerrainChunkVisual)]
pub struct TerrainChunkLogic(Entity);

#[derive(Debug, Clone, Component, Default, PartialEq, Eq, Hash)]
pub struct TerrainChunkMesh;

#[derive(Debug, Component)]
#[require(TerrainChunkMesh)]
pub struct TerrainChunkMeshHandle(pub Handle<Mesh>);
