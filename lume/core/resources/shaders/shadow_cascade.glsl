#ifndef SHADOW_CASCADE_GLSL
#define SHADOW_CASCADE_GLSL

#include "common.glsl"

const vec4 CULLED_VERTEX_POSITION = vec4(2.0, 2.0, 2.0, 1.0);

bool is_cascade_visible(uint cascade_mask, uint view_index) {
    return (cascade_mask & (1u << view_index)) != 0u;
}

vec4 cascade_clip_position(uint64_t shadow_cascades_buffer_device_address, uint view_index, vec4 world_position) {
    ShadowCascadesBuffer cascades = ShadowCascadesBuffer(shadow_cascades_buffer_device_address);

    return cascades.data[view_index].light_space_matrix * world_position;
}

#endif
