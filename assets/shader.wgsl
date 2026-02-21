#import bevy_pbr::{
    forward_io::{FragmentOutput, VertexOutput},
    mesh_bindings::mesh,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d, bindless_textures_2d_array}

#import bevy_pbr::pbr_bindings::{material_array, material_indices}

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
    // Fetch the material slot. We'll use this in turn to fetch the bindless
    // indices from `blended_pbr_indices`.
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;

    // Generate a `PbrInput` struct from the `StandardMaterial` bindings.
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let uv_transform = material_array[material_indices[slot].material].uv_transform;
    let uv = (uv_transform * vec3(in.uv, 1.0)).xy;
    let uv_b = (uv_transform * vec3(in.uv_b, 1.0)).xy;

    let indices = blended_pbr_indices[slot];

    let mask = textureSample(
        bindless_textures_2d[indices.mask],
        bindless_samplers_filtering[indices.mask_sampler],
        uv_b
    ).r;

    let base_color_array = bindless_textures_2d_array[indices.base_color_texture_index];
    let base_color_sampler = bindless_samplers_filtering[indices.base_color_sampler_index];

    let base_color_a = textureSample(
        base_color_array,
        base_color_sampler,
        uv,
        0
    );
    let base_color_b = textureSample(
        base_color_array,
        base_color_sampler,
        uv,
        1
    );


    let normal_array = bindless_textures_2d_array[indices.normal_texture_index];
    let normal_sampler = bindless_samplers_filtering[indices.normal_sampler_index];

    let normal_a = textureSample(
        normal_array,
        normal_sampler,
        uv,
        0
    ).rgb;
    let normal_b = textureSample(
        normal_array,
        normal_sampler,
        uv,
        1
    ).rgb;

    let arm_array = bindless_textures_2d_array[indices.arm_texture_index];
    let arm_sampler = bindless_samplers_filtering[indices.arm_sampler_index];

    let arm_a = textureSample(
        arm_array,
        arm_sampler,
        uv,
        0
    ).rgb;
    let arm_b = textureSample(
        arm_array,
        arm_sampler,
        uv,
        1
    ).rgb;

    pbr_input.material.base_color *= base_color_a * mask + base_color_b * (1.0 - mask);
    // Todo: this is not how normals are actually interpolated
    pbr_input.N = normalize((normal_a * mask + normal_b * (1.0 - mask)));
    pbr_input.material.perceptual_roughness *= arm_a.g * mask + arm_b.g * (1.0 - mask);
    pbr_input.material.metallic *= arm_a.b * mask + arm_b.b * (1.0 - mask);
    // Todo: I don't think this is how occlusion works lol
    pbr_input.material.base_color *= arm_a.r * mask + arm_b.r * (1.0 - mask);


    var out: FragmentOutput;
    // Apply lighting.
    out.color = apply_pbr_lighting(pbr_input);
    // Apply in-shader post processing (fog, alpha-premultiply, and also
    // tonemapping, debanding if the camera is non-HDR). Note this does not
    // include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
