#version 460
#extension GL_EXT_nonuniform_qualifier : enable

#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform sampler2D src[];

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
    uint id = nonuniformEXT(push_constants.src_texture);
    vec2 texel = 1.0 / vec2(textureSize(src[id], 0));

    vec3 result = texture(src[id], in_uv).rgb * 4.0;
    result += texture(src[id], in_uv + texel * vec2( 1.0,  0.0)).rgb * 2.0;
    result += texture(src[id], in_uv + texel * vec2(-1.0,  0.0)).rgb * 2.0;
    result += texture(src[id], in_uv + texel * vec2( 0.0,  1.0)).rgb * 2.0;
    result += texture(src[id], in_uv + texel * vec2( 0.0, -1.0)).rgb * 2.0;
    result += texture(src[id], in_uv + texel * vec2( 1.0,  1.0)).rgb;
    result += texture(src[id], in_uv + texel * vec2(-1.0,  1.0)).rgb;
    result += texture(src[id], in_uv + texel * vec2( 1.0, -1.0)).rgb;
    result += texture(src[id], in_uv + texel * vec2(-1.0, -1.0)).rgb;
    result *= (1.0 / 16.0);

    out_color = vec4(result, 1.0);
}
