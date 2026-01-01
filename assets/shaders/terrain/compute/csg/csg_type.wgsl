#define_import_path terrain::csg::csg_type

struct TerrainChunkCSGOperation {
    position: vec3<f32>,
    primitive_type: u32,
    // shape param
    shape: vec3<f32>,
    operate_type: u32,
}
