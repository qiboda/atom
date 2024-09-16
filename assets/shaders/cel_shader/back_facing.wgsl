#import bevy_pbr::mesh_functions::{ get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world }
#import bevy_pbr::view_transformations::{ position_world_to_clip, position_world_to_ndc, position_ndc_to_world, position_world_to_view, direction_world_to_view, direction_world_to_clip }


#import bevy_pbr::{ forward_io::{ VertexOutput, Vertex, FragmentOutput }}

@group(2) @binding(0)
var<uniform> stroke_color: vec4f;

@group(2) @binding(1)
var<uniform> stroke_width: f32;

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

    let tangent_world = mesh_normal_local_to_world(in.tangent.xyz, in.instance_index);
    let tangent_ndc = direction_world_to_clip(tangent_world);
    // 乘以w，之后out.position会除以w，这保证了屏幕空间中距离始终为tangent_ndc * stroke_width
    out.position.x += tangent_ndc.x * out.position.w * stroke_width;
    out.position.y += tangent_ndc.y * out.position.w * stroke_width;
    out.position.z += tangent_ndc.z * out.position.w * stroke_width;

    out.instance_index = in.instance_index;
    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = stroke_color;
    return out;
}
