#import bevy_pbr::{ forward_io::{FragmentOutput} } 
#import bevy_pbr::mesh_functions::{ get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world }
#import bevy_pbr::view_transformations::position_world_to_clip

#import trimap::biplanar::{calculate_biplanar_mapping, biplanar_texture, biplanar_texture_single, biplanar_texture_splatted}
#import trimap::triplanar::{calculate_triplanar_mapping, triplanar_normal_to_world, triplanar_normal_to_world_splatted}


@group(2) @binding(0)
var<uniform> lod: u32;

@group(2) @binding(1)
var color_texture: texture_2d<f32>;
@group(2) @binding(2)
var color_sampler: sampler;

@group(2) @binding(3)
var normal_texture: texture_2d<f32>;
@group(2) @binding(4)
var normal_sampler: sampler;

@group(2) @binding(5)
var roughness_texture: texture_2d<f32>;
@group(2) @binding(6)
var roughness_sampler: sampler;

@group(2) @binding(7)
var metallic_texture: texture_2d<f32>;
@group(2) @binding(8)
var metallic_sampler: sampler;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) material: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) world_position: vec4f,
    @location(1) world_normal: vec3f,
    @location(2) @interpolate(flat) material: u32,
    @location(3) @interpolate(flat) instance_index: u32,
}
 

@vertex
fn vertex(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = get_world_from_local(in.instance_index);
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(in.position, 1.0));
    out.clip_position = position_world_to_clip(out.world_position.xyz);

    out.world_normal = in.normal;
    // out.world_normal = mesh_normal_local_to_world(
    //     in.normal,
    //     in.instance_index
    // );

    out.material = in.material;
    out.instance_index = in.instance_index;
    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {

    let pos = in.world_position.xyz % 0.5;
    var bimap = calculate_biplanar_mapping(abs(pos.xyz), in.world_normal, 8.0);
    // bimap.ma_uv *= material.uv_scale;
    // bimap.me_uv *= material.uv_scale;
    // Triplanar is only used for normal mapping because the transitions between
    // planar projections look significantly better when there is high contrast
    // in lighting direction.
    var trimap = calculate_triplanar_mapping(in.world_position.xyz, in.world_normal, 8.0);
    // trimap.uv_x *= material.uv_scale;
    // trimap.uv_y *= material.uv_scale;
    // trimap.uv_z *= material.uv_scale;

    var out: FragmentOutput;

    let uv = abs(in.world_position.xy) % 1.0;

    let color = biplanar_texture_single(
        color_texture,
        color_sampler,
        bimap
    );

#ifdef COLOR_DEBUG
    let select_index = lod % 3;
    if select_index == 0 {
        out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);;
    } else if select_index == 1 {
        out.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    } else {
        out.color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }
#else ifdef NORMAL_DEBUG
    out.color = vec4<f32>(in.world_normal, 1.0);
#else 
    out.color = color;
#endif

    return out;
}
