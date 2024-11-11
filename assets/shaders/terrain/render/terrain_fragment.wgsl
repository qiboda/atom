#import bevy_pbr::{ pbr_types, pbr_types::{PbrInput}, pbr_functions, pbr_functions::{ SampleBias, apply_pbr_lighting, main_pass_post_lighting_processing }, lighting, mesh_bindings::mesh, mesh_view_bindings::view }

#import trimap::biplanar::{calculate_biplanar_mapping, biplanar_texture, biplanar_texture_single, biplanar_texture_splatted, BiplanarMapping}
#import trimap::triplanar::{calculate_triplanar_mapping, triplanar_normal_to_world, triplanar_normal_to_world_splatted, TriplanarMapping}

#import terrain::biome::TerrainType_Max
#import terrain::terrain_type::{TerrainVertexOutput}
#import terrain::terrain_bind_groups:: {
    terrain_material,
    base_color_texture,
    base_color_sampler,
    normal_map_texture,
    normal_map_sampler,
    metallic_roughness_texture,
    metallic_roughness_sampler,
    occlusion_texture,
    occlusion_sampler,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{prepass_io::{FragmentOutput}}
#else
#import bevy_pbr::{ forward_io::{FragmentOutput} }
#endif

// prepare a basic PbrInput from the vertex stage output, mesh binding and view binding
fn pbr_input_from_vertex_output(
    in: TerrainVertexOutput,
    is_front: bool,
    double_sided: bool,
) -> PbrInput {
    var pbr_input = pbr_types::pbr_input_new();

    pbr_input.flags = mesh[in.instance_index].flags;

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.V = pbr_functions::calculate_view(in.world_position, pbr_input.is_orthographic);
    pbr_input.frag_coord = in.clip_position;
    pbr_input.world_position = in.world_position;

    pbr_input.world_normal = pbr_functions::prepare_world_normal(
        in.world_normal,
        double_sided,
        is_front,
    );

    pbr_input.N = normalize(pbr_input.world_normal);

    return pbr_input;
}

fn fragment_debug(out: ptr<function, FragmentOutput>) {
#ifdef COLOR_DEBUG
    let select_index = terrain_material.lod % 8;
    (*out).color = vec4<f32>(
        f32((select_index & 0x001) == 0u),
        f32((select_index & 0x002) == 0u),
        f32((select_index & 0x004) == 0u),
        1.0
    );
#else ifdef NORMAL_DEBUG
    (*out).color = vec4<f32>(in.world_normal, 1.0);
#endif 
}

#ifdef PREPASS_PIPELINE
fn deferred_output(in: TerrainVertexOutput, pbr_input: PbrInput) -> FragmentOutput {
    var out: FragmentOutput;

#ifdef DEFERRED_PREPASS
    // gbuffer
    out.deferred = deferred_gbuffer_from_pbr_input(pbr_input);
    // lighting pass id (used to determine which lighting shader to run for the fragment)
    out.deferred_lighting_pass_id = pbr_input.material.deferred_lighting_pass_id;
#endif

    // normal if required
#ifdef NORMAL_PREPASS
    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
#endif
    // motion vectors if required
#ifdef MOTION_VECTOR_PREPASS
#ifdef MESHLET_MESH_MATERIAL_PASS
    out.motion_vector = in.motion_vector;
#else
    out.motion_vector = calculate_motion_vector(in.world_position, in.previous_world_position);
#endif
#endif

    return out;
}
#endif


fn apply_light(
    in: TerrainVertexOutput,
    out: ptr<function, FragmentOutput>,
    pbr_input: PbrInput,
    is_unlit: bool
) {
#ifdef PREPASS_PIPELINE
    // write the gbuffer, lighting pass id, and optionally normal and motion_vector textures
    *out = deferred_output(in, pbr_input);
#else
    if is_unlit {
        (*out).color = pbr_input.material.base_color;
    } else {
        (*out).color = apply_pbr_lighting(pbr_input);
    }
    (*out).color = main_pass_post_lighting_processing(pbr_input, (*out).color);
#endif
}

fn compute_color(in: TerrainVertexOutput) -> vec4f {
    var color = vec4<f32>(0.0);
    color += terrain_material.biome_colors[0].base_color * in.biome_weights_a.x;
    color += terrain_material.biome_colors[1].base_color * in.biome_weights_a.y;
    color += terrain_material.biome_colors[2].base_color * in.biome_weights_a.z;
    color += terrain_material.biome_colors[3].base_color * in.biome_weights_a.w;
    color += terrain_material.biome_colors[4].base_color * in.biome_weights_b.x;
    color += terrain_material.biome_colors[5].base_color * in.biome_weights_b.y;
    color += terrain_material.biome_colors[6].base_color * in.biome_weights_b.z;
    color += terrain_material.biome_colors[7].base_color * in.biome_weights_b.w;
    color += terrain_material.biome_colors[8].base_color * in.biome_weights_c.x;
    color += terrain_material.biome_colors[9].base_color * in.biome_weights_c.y;
    color += terrain_material.biome_colors[10].base_color * in.biome_weights_c.z;
    color += terrain_material.biome_colors[11].base_color * in.biome_weights_c.w;
    color += terrain_material.biome_colors[12].base_color * in.biome_weights_d.x;
    color += terrain_material.biome_colors[13].base_color * in.biome_weights_d.y;
    color += terrain_material.biome_colors[14].base_color * in.biome_weights_d.z;
    color += terrain_material.biome_colors[15].base_color * in.biome_weights_d.w;
    color += terrain_material.biome_colors[16].base_color * in.biome_weights_e.x;
    color += terrain_material.biome_colors[17].base_color * in.biome_weights_e.y;
    color += terrain_material.biome_colors[18].base_color * in.biome_weights_e.z;
    color += terrain_material.biome_colors[19].base_color * in.biome_weights_e.w;
    color += terrain_material.biome_colors[20].base_color * in.biome_weights_f.x;
    // color += terrain_material.biome_colors[21].base_color * in.biome_weights_f.y;
    // color += terrain_material.biome_colors[22].base_color * in.biome_weights_f.z;
    // color += terrain_material.biome_colors[23].base_color * in.biome_weights_f.w;
    return color;
}

fn sample_textures(pbr_input: ptr<function, PbrInput>, in: TerrainVertexOutput) {
    let texture_mapping_size = 8.0;
    let uv_scale = 1.0 / texture_mapping_size;

    let pos = in.world_position.xyz % texture_mapping_size;
    var bimap = calculate_biplanar_mapping(abs(pos), in.world_normal, 8.0);
    bimap.ma_uv *= uv_scale;
    bimap.me_uv *= uv_scale;

    var trimap = calculate_triplanar_mapping(abs(pos), in.world_normal, 8.0);
    trimap.uv_x *= uv_scale;
    trimap.uv_y *= uv_scale;
    trimap.uv_z *= uv_scale;

    (*pbr_input).material.base_color = compute_color(in);

//     let use_color_texture = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u;
//     if use_color_texture {
//         pbr_input.material.base_color *= biplanar_texture_splatted(
//             base_color_texture,
//             base_color_sampler,
//             in.biome_weights,
//             bimap
//         );
//     }
    
//     terrain_material.base_color

//     // 光照
//     let is_unlit = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 1u;
//     if is_unlit == false {
//         pbr_input.material.reflectance = terrain_material.reflectance;
//         pbr_input.material.attenuation_color = terrain_material.attenuation_color;
//         pbr_input.material.attenuation_distance = terrain_material.attenuation_distance;

//         let use_metallic_roughness_texture = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u;

//         // metallic and perceptual roughness
//         var metallic: f32 = terrain_material.metallic;
//         var perceptual_roughness: f32 = terrain_material.perceptual_roughness;
//         let roughness = lighting::perceptualRoughnessToRoughness(perceptual_roughness);
//         if use_metallic_roughness_texture {
//             let metallic_roughness = biplanar_texture_splatted(
//                 metallic_roughness_texture,
//                 metallic_roughness_sampler,
//                 in.biome_weights,
//                 bimap
//             );
//             // Sampling from GLTF standard channels for now
//             metallic *= metallic_roughness.b;
//             perceptual_roughness *= metallic_roughness.g;
//         }
//         pbr_input.material.metallic = metallic;
//         pbr_input.material.perceptual_roughness = perceptual_roughness;

//         var diffuse_occlusion: vec3<f32> = vec3(1.0);
//         var specular_occlusion: f32 = 1.0;
//         let use_occlusion_texture = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u;
//         if use_occlusion_texture {
//             diffuse_occlusion *= biplanar_texture_splatted(
//                 occlusion_texture,
//                 occlusion_sampler,
//                 in.biome_weights,
//                 bimap
//             ).r;
//         }
// #ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
//         let ssao = textureLoad(screen_space_ambient_occlusion_texture, vec2<i32>(in.position.xy), 0i).r;
//         let ssao_multibounce = gtao_multibounce(ssao, pbr_input.material.base_color.rgb);
//         diffuse_occlusion = min(diffuse_occlusion, ssao_multibounce);
//         // Use SSAO to estimate the specular occlusion.
//         // Lagarde and Rousiers 2014, "Moving Frostbite to Physically Based Rendering"
//         specular_occlusion = saturate(pow(NdotV + ssao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ssao);
// #endif
//         pbr_input.diffuse_occlusion = diffuse_occlusion;
//         pbr_input.specular_occlusion = specular_occlusion;

//         // 纹理法向量
//         let two_component_normal = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) != 0u;
//         let flip_normal_map_y = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) != 0u;

//         let normal = normalize(pbr_input.world_normal);
//         pbr_input.N = triplanar_normal_to_world_splatted(
//             two_component_normal,
//             flip_normal_map_y,
//             normal_map_texture,
//             normal_map_sampler,
//             terrain_material.flags,
//             in.biome_weights,
//             normal,
//             trimap,
//         );
//     }
}

@fragment
fn fragment(
    in: TerrainVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var out: FragmentOutput;

    let double_sided = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    var pbr_input = pbr_input_from_vertex_output(in, is_front, double_sided);
    pbr_input.material.flags = terrain_material.flags;

    sample_textures(&pbr_input, in);

    apply_light(in, &out, pbr_input, false);
    fragment_debug(&out);
    return out;
}
