#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput}
}

#import trimap::biplanar::{calculate_biplanar_mapping, biplanar_texture, biplanar_texture_single, biplanar_texture_splatted}
#import trimap::triplanar::{calculate_triplanar_mapping, triplanar_normal_to_world, triplanar_normal_to_world_splatted}

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

    out.color = color;

    return out;
}
