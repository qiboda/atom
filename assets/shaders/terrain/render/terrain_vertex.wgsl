#import bevy_pbr::mesh_functions::{ get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world }
#import bevy_pbr::view_transformations::position_world_to_clip

#import terrain::terrain_type::{ TerrainVertexInput, TerrainVertexOutput }
#import terrain::terrain_bind_groups::terrain_material
#import terrain::biome::TerrainType_MAX

@vertex
fn vertex(
    in: TerrainVertexInput,
) -> TerrainVertexOutput {
    var out: TerrainVertexOutput;

    let world_from_local = get_world_from_local(in.instance_index);
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(in.position, 1.0));
    out.clip_position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_normal_local_to_world(in.normal, in.instance_index);

    var biome_weights = array<f32, 24>();
    for (var i = 0u; i < TerrainType_MAX; i++) {
        if terrain_material.biome_colors[i].biome == in.biome {
            biome_weights[i] = 1.0;
        }
    }
    out.biome_weights_a = vec4f(biome_weights[0], biome_weights[1], biome_weights[2], biome_weights[3]);
    out.biome_weights_b = vec4f(biome_weights[4], biome_weights[5], biome_weights[6], biome_weights[7]);
    out.biome_weights_c = vec4f(biome_weights[8], biome_weights[9], biome_weights[10], biome_weights[11]);
    out.biome_weights_d = vec4f(biome_weights[12], biome_weights[13], biome_weights[14], biome_weights[15]);
    out.biome_weights_e = vec4f(biome_weights[16], biome_weights[17], biome_weights[18], biome_weights[19]);
    out.biome_weights_f = vec4f(biome_weights[20], biome_weights[21], biome_weights[22], biome_weights[23]);
    out.instance_index = in.instance_index;
    return out;
}
