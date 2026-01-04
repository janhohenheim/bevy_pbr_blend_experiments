#import bevy_pbr::{
    forward_io::{FragmentOutput, VertexOutput},
    mesh_bindings::mesh,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}

#import bevy_pbr::pbr_bindings::{material_array, material_indices}

struct BlendedPbrIndices {
    material: u32,
    mask: u32,
    mask_sampler: u32,
    blend_a: u32,
    blend_a_sampler: u32,
    blend_b: u32,
    blend_b_sampler: u32,
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

    let mask = textureSample(
        bindless_textures_2d[blended_pbr_indices[slot].mask],
        bindless_samplers_filtering[
            blended_pbr_indices[slot].mask_sampler
        ],
        uv
    );

    let blend_a = textureSample(
        bindless_textures_2d[blended_pbr_indices[slot].blend_a],
        bindless_samplers_filtering[
            blended_pbr_indices[slot].blend_a_sampler
        ],
        uv
    );
    let blend_b = textureSample(
        bindless_textures_2d[blended_pbr_indices[slot].blend_b],
        bindless_samplers_filtering[
            blended_pbr_indices[slot].blend_b_sampler
        ],
        uv
    );

    let blend = blend_b * mask + blend_a * (1.0 - mask);
    pbr_input.material.base_color *= blend;


    var out: FragmentOutput;
    // Apply lighting.
    out.color = apply_pbr_lighting(pbr_input);
    // Apply in-shader post processing (fog, alpha-premultiply, and also
    // tonemapping, debanding if the camera is non-HDR). Note this does not
    // include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
