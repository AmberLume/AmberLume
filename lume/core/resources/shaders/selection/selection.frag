#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../common.glsl"
#include "../bindings.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec4 out_color;

const uint ENTITY_NONE = 0xffffffffu;

uint entity_at(ivec2 coord, ivec2 size) {
    return texelFetch(
        graph_utextures[nonuniformEXT(push_constants.entity_id_texture)],
        clamp(coord, ivec2(0), size - 1),
        0
    ).r;
}

bool outline_nearby(ivec2 coord, ivec2 mask_size) {
    ivec2 first = clamp((coord - push_constants.radius) / push_constants.mask_scale, ivec2(0), mask_size - 1);
    ivec2 last = clamp((coord + push_constants.radius) / push_constants.mask_scale, ivec2(0), mask_size - 1);

    for (int y = first.y; y <= last.y; y++) {
        for (int x = first.x; x <= last.x; x++) {
            if (texelFetch(graph_utextures[nonuniformEXT(push_constants.mask_texture)], ivec2(x, y), 0).r != 0u) {
                return true;
            }
        }
    }

    return false;
}

void main() {
    ivec2 entity_id_size = textureSize(graph_utextures[nonuniformEXT(push_constants.entity_id_texture)], 0);
    ivec2 coord = clamp(ivec2(gl_FragCoord.xy * push_constants.entity_id_texel_scale), ivec2(0), entity_id_size - 1);

    EntityOutlineBuffer entity_outline_buffer = EntityOutlineBuffer(push_constants.entity_outline_buffer_device_address);
    vec2 jitter = SceneBuffer(push_constants.scene_buffer_device_address).data.main_camera.jitter;

    out_color = vec4(0.0);

    uint own_entity = entity_at(coord, entity_id_size);

    if (own_entity != ENTITY_NONE && entity_outline_buffer.data[own_entity].outline.a > 0.0) {
        return;
    }

    ivec2 mask_size = textureSize(graph_utextures[nonuniformEXT(push_constants.mask_texture)], 0);

    if (!outline_nearby(coord, mask_size)) {
        return;
    }

    int radius = push_constants.radius;

    float nearest = float((radius + 1) * (radius + 1));
    uint nearest_entity = ENTITY_NONE;

    for (int y = -radius; y <= radius; y++) {
        for (int x = -radius; x <= radius; x++) {
            vec2 delta = vec2(x, y) - jitter;
            float squared = dot(delta, delta);

            if (squared >= nearest) {
                continue;
            }

            uint entity_index = entity_at(coord + ivec2(x, y), entity_id_size);

            if (entity_index == ENTITY_NONE) {
                continue;
            }

            if (entity_outline_buffer.data[entity_index].outline.a <= 0.0) {
                continue;
            }

            nearest = squared;
            nearest_entity = entity_index;
        }
    }

    if (nearest_entity == ENTITY_NONE) {
        return;
    }

    vec4 color = entity_outline_buffer.data[nearest_entity].outline;

    float falloff = 1.0 - clamp(sqrt(nearest) / float(radius), 0.0, 1.0);

    out_color = vec4(color.rgb, color.a * falloff);
}
