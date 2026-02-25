#version 460
#extension GL_ARB_shader_draw_parameters : enable
#extension GL_EXT_nonuniform_qualifier : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform sampler2D textures[];

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec2 uv;
layout(location = 2) in flat uint draw_id;
layout(location = 0) out vec4 out_color;

void main() {
    DrawDataGpuData draw = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    SubmeshGpuData submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw.submesh_index];
    MaterialGpuData material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec3 normal = normalize(frag_normal);

    vec3 key_light = normalize(vec3(0.5, 1.0, 0.5));
    vec3 fill_light = normalize(vec3(-0.3, 0.2, 0.8));

    float key_diffuse = max(dot(normal, key_light), 0.0);
    float fill_diffuse = max(dot(normal, fill_light), 0.0);

    vec2 screen_uv = gl_FragCoord.xy / vec2(textureSize(textures[nonuniformEXT(push_constants.shadow_mask_descriptor_id)], 0));
    float shadow = texture(textures[push_constants.shadow_mask_descriptor_id], screen_uv).r;

    float ambient = 0.3;
    float lighting = ambient + (key_diffuse * 0.5 + fill_diffuse * 0.3) * shadow;

    vec4 diffuse_color = material.base_color;

    if (material.base_color_texture_index != 0xFFFFFFFF) {
        diffuse_color *= texture(textures[nonuniformEXT(material.base_color_texture_index)], uv);
    }

    out_color = vec4(diffuse_color.rgb * lighting, 1.0);
}
