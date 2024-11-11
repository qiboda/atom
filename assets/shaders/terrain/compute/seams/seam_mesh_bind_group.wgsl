#define_import_path terrain::seam_mesh_bind_group

#import terrain::voxel_type::{TerrainChunkInfo, TerrainChunkVertexInfo, TerrainChunkVerticesIndicesCount}

@group(0) @binding(0)
var<uniform> terrain_chunk_info: TerrainChunkInfo;

@group(0) @binding(1)
var<uniform> terrain_chunks_lod: array<vec4<u32>, 16>;

@group(0) @binding(2)
var<storage, read_write> mesh_vertices: array<TerrainChunkVertexInfo>;

@group(0) @binding(3)
var<storage, read_write> mesh_indices: array<u32>;

@group(0) @binding(4)
var<storage, read_write> mesh_vertex_map: array<u32>;

@group(0) @binding(5)
var<storage, read_write> mesh_vertices_indices_count: TerrainChunkVerticesIndicesCount;


