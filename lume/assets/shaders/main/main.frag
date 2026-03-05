#version 460
#extension GL_ARB_shader_draw_parameters : enable
#extension GL_EXT_nonuniform_qualifier : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform sampler2D textures[];

layout(location = 0) in mat3 in_TBN;
layout(location = 3) in vec2 uv;
layout(location = 4) in flat uint draw_id;
layout(location = 5) in vec3 world_pos;

layout(location = 0) out vec4 out_color;

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawDataGpuData draw = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    SubmeshGpuData submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw.submesh_index];
    MaterialGpuData material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec3 normal_sample = texture(textures[nonuniformEXT(material.normal_texture_index)], uv).rgb;
    vec3 local_normal = normal_sample * 2.0 - 1.0;
    vec3 normal = normalize(in_TBN * local_normal);

    vec2 screen_uv = gl_FragCoord.xy / vec2(textureSize(textures[nonuniformEXT(push_constants.shadow_mask_descriptor_id)], 0));
    float shadow = texture(textures[push_constants.shadow_mask_descriptor_id], screen_uv).r;

    vec4 occlution_roughness_metallic = texture(textures[nonuniformEXT(material.occlusion_roughness_metallic_texture_index)], uv);
    float ambient_occlusion = occlution_roughness_metallic.r;
    float roughness = occlution_roughness_metallic.g * material.roughness_factor;
    float metallic  = occlution_roughness_metallic.b * material.metallic_factor;

    vec4 albedo = texture(textures[nonuniformEXT(material.color_texture_index)], uv) * material.base_color_factor;
    albedo.rgb = pow(albedo.rgb, vec3(2.2));

    vec3 V = normalize(scene_buffer.data.main_camera.position - world_pos);
    vec3 L = normalize(-scene_buffer.data.light_direction);
    vec3 H = normalize(V + L);

    vec3 F0 = mix(vec3(0.04), albedo.rgb, metallic);

    float a2 = pow(roughness, 4.0);
    float NdotH = max(dot(normal, H), 0.0);
    float NdotV = max(dot(normal, V), 0.0);
    float NdotL = max(dot(normal, L), 0.0);

    float denom = (NdotH * NdotH * (a2 - 1.0) + 1.0);
    float NDF = a2 / (3.14159 * denom * denom);

    float k = pow(roughness + 1.0, 2.0) / 8.0;
    float G = (NdotV / (NdotV * (1.0 - k) + k)) * (NdotL / (NdotL * (1.0 - k) + k));

    vec3 F = F0 + (1.0 - F0) * pow(clamp(1.0 - max(dot(H, V), 0.0), 0.0, 1.0), 5.0);

    vec3 specular = (NDF * G * F) / (4.0 * NdotV * NdotL + 0.0001);
    vec3 kD = (vec3(1.0) - F) * (1.0 - metallic);
    vec3 Lo = (kD * albedo.rgb / 3.14159 + specular) * NdotL * shadow;

    vec3 ambient = vec3(0.05) * albedo.rgb * ambient_occlusion;
    vec3 color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    out_color = vec4(color, albedo.a);
}
