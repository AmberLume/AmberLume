#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

const vec3 REC709_LUMINANCE = vec3(0.2126, 0.7152, 0.0722);
const float MIDDLE_GREY = 0.18;

vec3 color_grade(vec3 color, float saturation, float contrast, vec3 offset, vec3 gamma, vec3 gain) {
    float luminance = dot(color, REC709_LUMINANCE);
    color = max(mix(vec3(luminance), color, saturation), 0.0);
    color = MIDDLE_GREY * pow(color / MIDDLE_GREY, vec3(contrast));
    color = pow(color, 1.0 / gamma);

    return max(color * gain + offset, 0.0);
}

const float PBR_NEUTRAL_F90 = 0.04;
const float PBR_NEUTRAL_COMPRESSION_START = 0.8;
const float PBR_NEUTRAL_DESATURATION = 0.15;

vec3 pbr_neutral(vec3 color, float display_peak) {
    float lowest = min(color.r, min(color.g, color.b));
    float offset = lowest < 2.0 * PBR_NEUTRAL_F90 ? lowest - lowest * lowest / (4.0 * PBR_NEUTRAL_F90) : PBR_NEUTRAL_F90;
    color -= offset;

    float peak = max(color.r, max(color.g, color.b));
    float start_compression = PBR_NEUTRAL_COMPRESSION_START * display_peak - PBR_NEUTRAL_F90;
    if (peak < start_compression) {
        return color;
    }

    float shoulder = display_peak - start_compression;
    float compressed_peak = display_peak - shoulder * shoulder / (peak + shoulder - start_compression);
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
    vec3 offset = vec3(push_constants.offset[0], push_constants.offset[1], push_constants.offset[2]);
    vec3 gamma = vec3(push_constants.gamma[0], push_constants.gamma[1], push_constants.gamma[2]);
    vec3 gain = vec3(push_constants.gain[0], push_constants.gain[1], push_constants.gain[2]);
    color = color_grade(color, push_constants.saturation, push_constants.contrast, offset, gamma, gain);
    color = pbr_neutral(color, push_constants.display_peak);

    if (push_constants.hdr == 1u) {
        color *= push_constants.paper_white;
    }

    out_color = vec4(color, 1.0);
}
