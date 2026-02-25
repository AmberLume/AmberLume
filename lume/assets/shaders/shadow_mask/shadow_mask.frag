#version 460

#extension GL_EXT_nonuniform_qualifier : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;
layout(location = 0) out float out_mask;

layout(set = 0, binding = 0) uniform sampler2D textures[];
layout(set = 0, binding = 3) uniform sampler2DArrayShadow shadow_arrays[];

void main() {
    float near = push_constants.camera_near;
    float far = push_constants.camera_far;
    float bias = push_constants.bias;
    int pcf_radius = push_constants.pcf_radius;

    ShadowBuffer shadow = ShadowBuffer(push_constants.shadow_buffer_device_address);

    float depth = texture(textures[nonuniformEXT(push_constants.depth_descriptor_id)], in_uv).r;

    if (depth >= 1.0) {
        out_mask = 1.0;
        return;
    }

    float view_z = (near * far) / (far - depth * (far - near));

    uint cascade_index = 0;
    for (uint i = 0; i < push_constants.cascade_count - 1; ++i) {
        if (view_z > shadow.cascades[i].split) {
            cascade_index = i + 1;
        }
    }

    vec4 screen_position = vec4(in_uv * 2.0 - 1.0, depth, 1.0);
    vec4 shadow_position = shadow.cascades[cascade_index].screen_to_light * screen_position;
    vec3 projection = shadow_position.xyz / shadow_position.w;

    float shadow_value = 0.0;
    float samples = 0.0;

    vec2 texel_size = vec2(1.0) / vec2(textureSize(shadow_arrays[nonuniformEXT(push_constants.global_shadow_descriptor_id)], 0).xy);

    for (int x = -pcf_radius; x <= pcf_radius; x++) {
        for (int y = -pcf_radius; y <= pcf_radius; y++) {
            vec4 coords = vec4(
                projection.xy * 0.5 + 0.5 + vec2(x, y) * texel_size,
                float(cascade_index),
                projection.z - bias
            );
            shadow_value += texture(shadow_arrays[nonuniformEXT(push_constants.global_shadow_descriptor_id)], coords);
            samples += 1.0;
        }
    }

    out_mask = shadow_value / samples;
}
