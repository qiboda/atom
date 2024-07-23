use std::ops::Deref;

use autoincrement::{AutoIncrement, Incremental};
use bevy::prelude::*;

use crate::isosurface::dc::octree::address::NodeAddress;

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TerrainChunkState {
    CreateMainMesh,
    WaitToCreateSeam,
    CreateSeamMesh,
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

    /// 递增id，返回旧的id。
    pub fn pull(&mut self) -> SeamMeshId {
        self.0.pull()
    }

    // 递增id，并返回一个递增了的id。
    pub fn gen(&mut self) -> SeamMeshId {
        self.pull();
        self.current()
    }

    pub fn current(&self) -> SeamMeshId {
        self.0.current()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Reflect, Clone, Copy, Component, Default)]
pub struct TerrainChunkAddress(NodeAddress);

impl Deref for TerrainChunkAddress {
    type Target = NodeAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TerrainChunkAddress {
    pub fn new(address: NodeAddress) -> Self {
        Self(address)
    }
}

impl From<&NodeAddress> for TerrainChunkAddress {
    fn from(value: &NodeAddress) -> Self {
        Self(*value)
    }
}

impl From<NodeAddress> for TerrainChunkAddress {
    fn from(value: NodeAddress) -> Self {
        Self(value)
    }
}
