
#import bevy_layered_materials::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_types::PbrInput
}
#import bevy_pbr::mesh_view_bindings::globals;

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
}
#import bevy_layered_materials::{
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d, bindless_textures_2d_array}


struct BlendedPbrIndices {
    material: u32,
    mask: u32,
    mask_sampler: u32,
    base_color_texture_index: u32,
    base_color_sampler_index: u32,
    normal_texture_index: u32,
    normal_sampler_index: u32,
    arm_texture_index: u32,
    arm_sampler_index: u32,
}

struct BlendedPbr {
    // unused for now :)
    strength: f32,
}


@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<storage> blended_pbr_indices:
    array<BlendedPbrIndices>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var<storage> blended_pbr:
    array<BlendedPbr>;


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input: PbrInput;
    let value = (sin(globals.time) + 1) / 2;
    var pbr_input1 = pbr_input_from_standard_material(in, is_front, 0);
    var pbr_input2 = pbr_input_from_standard_material(in, is_front, 1);
    if value > in.uv.x {
        pbr_input = pbr_input1;
    } else {
        pbr_input = pbr_input2;
        pbr_input.N = pbr_input1.N;
    }

    // we can optionally modify the input before lighting and alpha_discard is applied
    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    // out.color = out.color * 2.0;
#endif

    return out;
}
/*
@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let uv_transform = material_array[material_indices[slot].material].uv_transform;
    var uv = (uv_transform * vec3(in.uv, 1.0)).xy;
    var uv_b = (uv_transform * vec3(in.uv_b, 1.0)).xy;

    let indices = blended_pbr_indices[slot];
    let mask_texture = bindless_textures_2d[indices.mask];
    let mask_sampler = bindless_samplers_filtering[indices.mask_sampler];

    // Base Color
    pbr_input.material.base_color *= laplace_blend(
        bindless_textures_2d_array[indices.base_color_texture_index],
      bindless_samplers_filtering[indices.base_color_sampler_index],
      uv,
      mask_texture,
      mask_sampler,
      uv_b
    );

    // Normals
    let blended_normal_raw = laplace_blend(
        bindless_textures_2d_array[indices.normal_texture_index],
        bindless_samplers_filtering[indices.normal_sampler_index],
        uv,
        mask_texture,
        mask_sampler,
        uv_b
    ).rgb;
    let TBN = bevy_pbr::pbr_functions::calculate_tbn_mikktspace(
        pbr_input.world_normal,
        in.world_tangent,
    );

    let double_sided = (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;
    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        TBN,
        double_sided,
        is_front,
        blended_normal_raw.rgb,
    );

    // Linear
    let arm = laplace_blend(
         bindless_textures_2d_array[indices.arm_texture_index],
        bindless_samplers_filtering[indices.arm_sampler_index],
        uv,
        mask_texture,
        mask_sampler,
        uv_b
    );

    pbr_input.material.perceptual_roughness *= arm.g;
    pbr_input.material.metallic *= arm.b;
    pbr_input.diffuse_occlusion *= arm.r;


    // Apply PBR stuffs
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
*/

fn laplace_blend(texture_array: texture_2d_array<f32>, texture_sampler: sampler, uv: vec2<f32>, mask_texture: texture_2d<f32>, mask_sampler: sampler, uv_b: vec2<f32>) -> vec4<f32> {
    const NUM_LEVELS: i32 = 4;

    var tex0_levels: array<vec4<f32>, NUM_LEVELS + 1>;
    var tex1_levels: array<vec4<f32>, NUM_LEVELS + 1>;
    var mask_levels: array<f32, NUM_LEVELS + 1>;

    for (var i: i32 = 0; i < NUM_LEVELS + 1; i += 1) {
        let lod = f32(i);
        tex0_levels[i] = textureSampleLevel(texture_array, texture_sampler, uv, 0, lod);
        tex1_levels[i] = textureSampleLevel(texture_array, texture_sampler, uv, 1, lod);
        mask_levels[i] = textureSampleLevel(mask_texture, mask_sampler, uv_b, lod).r;
    }

    var blended: vec4<f32> = vec4<f32>(0.0);

    for (var i: i32 = 0; i < NUM_LEVELS; i += 1) {
        let tex0_laplace = tex0_levels[i] - tex0_levels[i + 1];
        let tex1_laplace = tex1_levels[i] - tex1_levels[i + 1];
        blended += tex0_laplace * (1.0 - mask_levels[i]) +
                tex1_laplace * mask_levels[i];
    }

    // Gaussian level.
    let tex0_gauss = tex0_levels[NUM_LEVELS];
    let tex1_gauss = tex1_levels[NUM_LEVELS];
    blended += tex0_gauss * (1.0 - mask_levels[NUM_LEVELS]) +
            tex1_gauss * mask_levels[NUM_LEVELS];
    return blended;
}
