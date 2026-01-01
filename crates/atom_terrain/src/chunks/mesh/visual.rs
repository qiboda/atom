use bevy::prelude::*;

#[derive(Debug, Component, Clone, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = TerrainChunkLogic)]
pub struct TerrainChunkVisual(Entity);

#[derive(Debug, Component, Clone, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = TerrainChunkVisual)]
pub struct TerrainChunkLogic(Entity);

#[derive(Debug, Clone, Component, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct TerrainChunkMesh;
