#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec4 out_color;

const vec3 BOX_VERTS[24] = vec3[](
    vec3(-1, -1, -1), vec3( 1, -1, -1),
    vec3( 1, -1, -1), vec3( 1, -1,  1),
    vec3( 1, -1,  1), vec3(-1, -1,  1),
    vec3(-1, -1,  1), vec3(-1, -1, -1),

    vec3(-1,  1, -1), vec3( 1,  1, -1),
    vec3( 1,  1, -1), vec3( 1,  1,  1),
    vec3( 1,  1,  1), vec3(-1,  1,  1),
    vec3(-1,  1,  1), vec3(-1,  1, -1),

    vec3(-1, -1, -1), vec3(-1,  1, -1),
    vec3( 1, -1, -1), vec3( 1,  1, -1),
    vec3( 1, -1,  1), vec3( 1,  1,  1),
    vec3(-1, -1,  1), vec3(-1,  1,  1)
);

vec3 get_box_vertex(uint vertex_id, vec3 half_extents) {
    return BOX_VERTS[vertex_id] * half_extents;
}

vec3 get_collider_vertex(uint vertex_id, uint shape_type, vec4 params) {
    switch (shape_type) {
        case SHAPE_BOX:
            return get_box_vertex(vertex_id, params.xyz);
        default:
            return get_box_vertex(vertex_id, params.xyz);
    }
}

void main() {
    ColliderGpuData collider = ColliderBuffer(push_constants.collider_buffer_device_address).data[gl_InstanceIndex];

    vec3 local_position = get_collider_vertex(gl_VertexIndex, collider.shape_type, collider.half_extents);
    vec4 world_position = collider.transform_matrix * vec4(local_position, 1.0);

    gl_Position = push_constants.projection_matrix * world_position;
    out_color = collider.color;
}
