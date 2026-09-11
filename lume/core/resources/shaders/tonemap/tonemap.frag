#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

const float PBR_NEUTRAL_START_COMPRESSION = 0.8 - 0.04;
const float PBR_NEUTRAL_DESATURATION = 0.15;

vec3 pbr_neutral(vec3 color) {
    float lowest = min(color.r, min(color.g, color.b));
    float offset = lowest < 0.08 ? lowest - 6.25 * lowest * lowest : 0.04;
    color -= offset;

    float peak = max(color.r, max(color.g, color.b));
    if (peak < PBR_NEUTRAL_START_COMPRESSION) {
        return color;
    }

    float shoulder = 1.0 - PBR_NEUTRAL_START_COMPRESSION;
    float compressed_peak = 1.0 - shoulder * shoulder / (peak + shoulder - PBR_NEUTRAL_START_COMPRESSION);
    color *= compressed_peak / peak;

    float desaturation = 1.0 - 1.0 / (PBR_NEUTRAL_DESATURATION * (peak - compressed_peak) + 1.0);
    return mix(color, vec3(compressed_peak), desaturation);
}

vec3 sample_input(uint tex_id, vec2 uv) {
    return texture(sampler2D(graph_textures[nonuniformEXT(tex_id)], samplers[SAMPLER_LINEAR_CLAMP]), uv).rgb;
}

vec3 sharpen(uint tex_id, vec2 uv) {
    vec3 center = sample_input(tex_id, uv);

    if (push_constants.sharpness <= 0.0) {
        return center;
    }

    vec2 texel = 1.0 / vec2(textureSize(graph_textures[nonuniformEXT(tex_id)], 0));

    vec3 up = sample_input(tex_id, uv + vec2(0.0, -texel.y));
    vec3 down = sample_input(tex_id, uv + vec2(0.0, texel.y));
    vec3 left = sample_input(tex_id, uv + vec2(-texel.x, 0.0));
    vec3 right = sample_input(tex_id, uv + vec2(texel.x, 0.0));

    vec3 lo = min(center, min(min(up, down), min(left, right)));
    vec3 hi = max(center, max(max(up, down), max(left, right)));

    vec3 sharpened = center + (center * 4.0 - (up + down + left + right)) * (push_constants.sharpness * 0.4);

    return clamp(sharpened, lo, hi);
}

void main() {
    uint scene_id = nonuniformEXT(push_constants.input_texture);
    vec3 color = sharpen(scene_id, in_uv);

    if (push_constants.bloom_intensity > 0.0) {
        vec3 bloom = texture(sampler2D(graph_textures[nonuniformEXT(push_constants.bloom_texture)], samplers[SAMPLER_LINEAR_CLAMP]), in_uv).rgb;
        color += bloom * push_constants.bloom_intensity;
    }

    color *= push_constants.exposure;
    color = pbr_neutral(color);

    if (push_constants.hdr == 1u) {
        color *= push_constants.paper_white;
    }

    out_color = vec4(color, 1.0);
}
