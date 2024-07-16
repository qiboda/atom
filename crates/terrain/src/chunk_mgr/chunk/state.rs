use autoincrement::{AsyncIncremental, AutoIncrement, Incremental};
use bevy::prelude::*;

use super::chunk_lod::LodType;

#[derive(Debug, Component)]
pub enum TerrainChunkState {
    CreateMainMesh(LodType),
    WaitToCreateSeam(LodType),
    CreateSeamMesh(SeamMeshId),
    Done,
}

#[derive(Incremental, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct SeamMeshId(u64);

#[derive(Component, Debug, Clone)]
pub struct SeamMeshIdGenerator(AutoIncrement<SeamMeshId>);

impl Default for SeamMeshIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SeamMeshIdGenerator {
    pub fn new() -> Self {
        Self(SeamMeshId::init())
    }

    pub fn next(&mut self) -> SeamMeshId {
        self.0.pull()
    }

    pub fn current(&self) -> SeamMeshId {
        self.0.current()
    }
}
