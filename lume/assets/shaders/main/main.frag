#version 460
#extension GL_ARB_shader_draw_parameters : enable
#extension GL_EXT_nonuniform_qualifier : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform sampler2D textures[];

layout(location = 0) in mat3 in_TBN;
layout(location = 3) in vec2 uv;
layout(location = 4) in flat uint draw_id;

layout(location = 0) out vec4 out_color;

void main() {
    DrawDataGpuData draw = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    SubmeshGpuData submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw.submesh_index];
    MaterialGpuData material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec3 normal_sample = texture(textures[nonuniformEXT(material.normal_texture_index)], uv).rgb;
    vec3 local_normal = normal_sample * 2.0 - 1.0;
    vec3 normal = normalize(in_TBN * local_normal);

    vec3 key_light = normalize(-push_constants.light_direction);

    float key_diffuse = max(dot(normal, key_light), 0.0);

    vec2 screen_uv = gl_FragCoord.xy / vec2(textureSize(textures[nonuniformEXT(push_constants.shadow_mask_descriptor_id)], 0));
    float shadow = texture(textures[push_constants.shadow_mask_descriptor_id], screen_uv).r;

    float ambient = 0.2;
    float lighting = ambient + (key_diffuse * shadow);

    vec4 diffuse_color = texture(textures[nonuniformEXT(material.color_texture_index)], uv) * material.base_color_factor;

    out_color = vec4(diffuse_color.rgb * lighting, diffuse_color.a);
}
