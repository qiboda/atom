#import bevy_pbr::mesh_functions::{ get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world }
#import bevy_pbr::view_transformations::position_world_to_clip

#import terrain::terrain_type::{ TerrainVertexInput, TerrainVertexOutput }

// The builtin one didn't work in webgl.
// "'unpackUnorm4x8' : no matching overloaded function found"
// https://github.com/gfx-rs/naga/issues/2006
fn unpack_unorm4x8_(v: u32) -> vec4<f32> {
    return vec4(
        f32(v & 0xFFu),
        f32((v >> 8u) & 0xFFu),
        f32((v >> 16u) & 0xFFu),
        f32((v >> 24u) & 0xFFu)
    ) / 255.0;
}

@vertex
fn vertex(
    in: TerrainVertexInput,
) -> TerrainVertexOutput {
    var out: TerrainVertexOutput;

    let world_from_local = get_world_from_local(in.instance_index);
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(in.position, 1.0));
    out.clip_position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_normal_local_to_world(in.normal, in.instance_index);

    out.material_weights = unpack_unorm4x8_(in.material);
    out.material_weights.x = 0.0;
    out.material_weights.y = 0.0;
    out.material_weights.z = 1.0;
    out.material_weights.w = 0.0;
    out.instance_index = in.instance_index;
    return out;
}
