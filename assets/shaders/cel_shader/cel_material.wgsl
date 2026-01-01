#import bevy_pbr::mesh_functions::{ get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world }
#import bevy_pbr::view_transformations::{ position_world_to_clip, position_world_to_ndc, position_ndc_to_world, position_world_to_view, direction_world_to_view, direction_world_to_clip }

#import bevy_pbr::{ forward_io::{ VertexOutput, Vertex, FragmentOutput }}

@group(2) @binding(0)
var<uniform> base_color: vec4f;

@vertex
fn vertex(
    in: Vertex,
) -> VertexOutput {

    var out: VertexOutput;

    let world_from_local = get_world_from_local(in.instance_index);
    out.world_normal = mesh_normal_local_to_world(in.normal, in.instance_index);

    out.world_normal = normalize(out.world_normal);

    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(in.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);

    out.instance_index = in.instance_index;
    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = base_color;

    return out;
}
