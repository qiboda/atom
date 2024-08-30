#import bevy_pbr::{ pbr_types, pbr_functions, pbr_functions::{ SampleBias, apply_pbr_lighting, main_pass_post_lighting_processing }, lighting, mesh_bindings::mesh, mesh_view_bindings::view }

#import trimap::biplanar::{calculate_biplanar_mapping, biplanar_texture, biplanar_texture_single, biplanar_texture_splatted, BiplanarMapping}
#import trimap::triplanar::{calculate_triplanar_mapping, triplanar_normal_to_world, triplanar_normal_to_world_splatted, TriplanarMapping}

#import terrain::terrain_type::{TerrainVertexOutput, TerrainFragmentOutput}
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

// prepare a basic PbrInput from the vertex stage output, mesh binding and view binding
fn pbr_input_from_vertex_output(
    in: TerrainVertexOutput,
    is_front: bool,
    double_sided: bool,
) -> pbr_types::PbrInput {
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

fn fragment_debug(out: ptr<function, TerrainFragmentOutput>) {
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

fn apply_light(out: ptr<function, TerrainFragmentOutput>, pbr_input: pbr_types::PbrInput, is_unlit: bool) {
#ifdef PREPASS_PIPELINE
    // write the gbuffer, lighting pass id, and optionally normal and motion_vector textures
    *out = deferred_output(in, *pbr_input);
#else
    if is_unlit {
        (*out).color = pbr_input.material.base_color;
    } else {
        (*out).color = apply_pbr_lighting(pbr_input);
    }
    (*out).color = main_pass_post_lighting_processing(pbr_input, (*out).color);
#endif
}

@fragment
fn fragment(
    in: TerrainVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> TerrainFragmentOutput {
    var out: TerrainFragmentOutput;

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

    let double_sided = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    var pbr_input = pbr_input_from_vertex_output(in, is_front, double_sided);
    pbr_input.material.flags = terrain_material.flags;
    pbr_input.material.base_color = terrain_material.base_color;

    let use_color_texture = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u;
    if use_color_texture {
        pbr_input.material.base_color *= biplanar_texture_splatted(
            base_color_texture,
            base_color_sampler,
            in.material_weights,
            bimap
        );
    }

    // 光照
    let is_unlit = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 1u;
    if is_unlit == false {
        pbr_input.material.reflectance = terrain_material.reflectance;
        pbr_input.material.attenuation_color = terrain_material.attenuation_color;
        pbr_input.material.attenuation_distance = terrain_material.attenuation_distance;

        let use_metallic_roughness_texture = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u;

        // metallic and perceptual roughness
        var metallic: f32 = terrain_material.metallic;
        var perceptual_roughness: f32 = terrain_material.perceptual_roughness;
        let roughness = lighting::perceptualRoughnessToRoughness(perceptual_roughness);
        if use_metallic_roughness_texture {
            let metallic_roughness = biplanar_texture_splatted(
                metallic_roughness_texture,
                metallic_roughness_sampler,
                in.material_weights,
                bimap
            );
            // Sampling from GLTF standard channels for now
            metallic *= metallic_roughness.b;
            perceptual_roughness *= metallic_roughness.g;
        }
        pbr_input.material.metallic = metallic;
        pbr_input.material.perceptual_roughness = perceptual_roughness;

        var diffuse_occlusion: vec3<f32> = vec3(1.0);
        var specular_occlusion: f32 = 1.0;
        let use_occlusion_texture = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u;
        if use_occlusion_texture {
            diffuse_occlusion *= biplanar_texture_splatted(
                occlusion_texture,
                occlusion_sampler,
                in.material_weights,
                bimap
            ).r;
        }
#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
        let ssao = textureLoad(screen_space_ambient_occlusion_texture, vec2<i32>(in.position.xy), 0i).r;
        let ssao_multibounce = gtao_multibounce(ssao, pbr_input.material.base_color.rgb);
        diffuse_occlusion = min(diffuse_occlusion, ssao_multibounce);
        // Use SSAO to estimate the specular occlusion.
        // Lagarde and Rousiers 2014, "Moving Frostbite to Physically Based Rendering"
        specular_occlusion = saturate(pow(NdotV + ssao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ssao);
#endif
        pbr_input.diffuse_occlusion = diffuse_occlusion;
        pbr_input.specular_occlusion = specular_occlusion;

        // 纹理法向量
        let two_component_normal = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) != 0u;
        let flip_normal_map_y = (terrain_material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) != 0u;

        let normal = normalize(pbr_input.world_normal);
        pbr_input.N = triplanar_normal_to_world_splatted(
            two_component_normal,
            flip_normal_map_y,
            normal_map_texture,
            normal_map_sampler,
            terrain_material.flags,
            in.material_weights,
            normal,
            trimap,
        );
    }

    apply_light(&out, pbr_input, is_unlit);
    fragment_debug(&out);
    return out;
}
