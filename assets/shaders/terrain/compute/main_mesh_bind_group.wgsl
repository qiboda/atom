#define_import_path terrain::main_mesh_bind_group

#import terrain::voxel_type::{TerrainChunkInfo, VoxelEdgeCrossPoint, TerrainChunkVertexInfo, TerrainChunkVerticesIndicesCount, TerrainMapConfig}
#import terrain::csg::csg_type::TerrainChunkCSGOperation

@group(0) @binding(0)
var<uniform> terrain_chunk_info: TerrainChunkInfo;

@group(0) @binding(1)
var<storage, read_write> voxel_vertex_values: array<f32>;

@group(0) @binding(2)
var<storage, read_write> voxel_cross_points: array<VoxelEdgeCrossPoint>;

@group(0) @binding(3)
var<storage, read_write> mesh_vertices: array<TerrainChunkVertexInfo>;

@group(0) @binding(4)
var<storage, read_write> mesh_indices: array<u32>;

@group(0) @binding(5)
var<storage, read_write> mesh_vertex_map: array<u32>;

@group(0) @binding(6)
var<storage, read_write> mesh_vertices_indices_count: TerrainChunkVerticesIndicesCount;

@group(1) @binding(0)
var<uniform> csg_info: u32;

@group(1) @binding(1)
var<storage, read> csg_operations: array<TerrainChunkCSGOperation>;

@group(2) @binding(0)
var<uniform> map_config: TerrainMapConfig;

@group(2) @binding(1)
var height_map_texture: texture_2d<f32>;
@group(2) @binding(2)
var height_map_sampler: sampler;

@group(2) @binding(3)
var biome_map_texture: texture_2d<u32>;
@group(2) @binding(4)
var biome_map_sampler: sampler;
