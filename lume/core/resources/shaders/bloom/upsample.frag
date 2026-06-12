#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
    uint id = nonuniformEXT(push_constants.src_texture);
    vec2 texel = 1.0 / vec2(textureSize(graph_textures[id], 0));

    vec3 result = texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv).rgb * 4.0;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 1.0,  0.0)).rgb * 2.0;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-1.0,  0.0)).rgb * 2.0;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 0.0,  1.0)).rgb * 2.0;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 0.0, -1.0)).rgb * 2.0;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 1.0,  1.0)).rgb;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-1.0,  1.0)).rgb;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2( 1.0, -1.0)).rgb;
    result += texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]),in_uv + texel * vec2(-1.0, -1.0)).rgb;
    result *= (1.0 / 16.0);

    out_color = vec4(result, 1.0);
}
