#version 460

#include "../bindings.glsl"
#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 uv;
layout(location = 1) in flat uint draw_id;

layout(location = 0) out vec4 out_transmittance;

void main() {
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];
    Material material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec4 albedo = texture(sampler2D(textures[nonuniformEXT(material.color_texture_index)], samplers[SAMPLER_LINEAR_CLAMP]), uv) * material.base_color_factor;

    out_transmittance = vec4(1.0 - albedo.a);
}
