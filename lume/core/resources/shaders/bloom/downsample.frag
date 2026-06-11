#version 460
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_samplerless_texture_functions : enable

#include "push_constants.glsl"
#include "../samplers.glsl"

layout(set = 0, binding = 0) uniform texture2D src[];

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

float karis_weight(vec3 c) {
    float luma = dot(c, vec3(0.2126, 0.7152, 0.0722));
    return 1.0 / (1.0 + luma);
}

vec3 prefilter(vec3 c, float threshold) {
    float knee = threshold * 0.5 + 0.0001;
    float brightness = max(c.r, max(c.g, c.b));
    float soft = clamp(brightness - threshold + knee, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee);
    float contribution = max(soft, brightness - threshold) / max(brightness, 0.0001);
    return c * contribution;
}

void main() {
    uint id = nonuniformEXT(push_constants.src_texture);
    vec2 texel = 1.0 / vec2(textureSize(src[id], 0));

    vec3 a = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-2.0,  2.0)).rgb;
    vec3 b = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 0.0,  2.0)).rgb;
    vec3 c = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 2.0,  2.0)).rgb;
    vec3 d = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-2.0,  0.0)).rgb;
    vec3 e = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 0.0,  0.0)).rgb;
    vec3 f = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 2.0,  0.0)).rgb;
    vec3 g = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-2.0, -2.0)).rgb;
    vec3 h = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 0.0, -2.0)).rgb;
    vec3 i = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 2.0, -2.0)).rgb;
    vec3 j = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-1.0,  1.0)).rgb;
    vec3 k = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 1.0,  1.0)).rgb;
    vec3 l = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-1.0, -1.0)).rgb;
    vec3 m = texture(sampler2D(src[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 1.0, -1.0)).rgb;

    vec3 result;
    if (push_constants.karis == 1u) {
        float t = push_constants.threshold;
        a = prefilter(a, t);
        b = prefilter(b, t);
        c = prefilter(c, t);
        d = prefilter(d, t);
        e = prefilter(e, t);
        f = prefilter(f, t);
        g = prefilter(g, t);
        h = prefilter(h, t);
        i = prefilter(i, t);
        j = prefilter(j, t);
        k = prefilter(k, t);
        l = prefilter(l, t);
        m = prefilter(m, t);

        vec3 group_center = (j + k + l + m) * 0.25;
        vec3 group_tl = (a + b + d + e) * 0.25;
        vec3 group_tr = (b + c + e + f) * 0.25;
        vec3 group_bl = (d + e + g + h) * 0.25;
        vec3 group_br = (e + f + h + i) * 0.25;

        float w_center = karis_weight(group_center) * 0.5;
        float w_tl = karis_weight(group_tl) * 0.125;
        float w_tr = karis_weight(group_tr) * 0.125;
        float w_bl = karis_weight(group_bl) * 0.125;
        float w_br = karis_weight(group_br) * 0.125;

        result = group_center * w_center
            + group_tl * w_tl
            + group_tr * w_tr
            + group_bl * w_bl
            + group_br * w_br;
        result /= (w_center + w_tl + w_tr + w_bl + w_br);
    } else {
        result = e * 0.125;
        result += (a + c + g + i) * 0.03125;
        result += (b + d + f + h) * 0.0625;
        result += (j + k + l + m) * 0.125;
    }

    out_color = vec4(result, 1.0);
}
