#version 460
#extension GL_ARB_shader_draw_parameters : enable
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "../common.glsl"
#include "../ibl.glsl"
#include "push_constants.glsl"

layout(location = 0) in mat3 in_TBN;
layout(location = 3) in vec2 uv;
layout(location = 4) in flat uint draw_id;
layout(location = 5) in vec3 world_pos;

layout(location = 0) out vec4 out_color;
layout(location = 1) out uint out_entity_index;

const float MIN_ROUGHNESS = 0.045;

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];
    Material material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec3 normal_sample = texture(sampler2D(textures[nonuniformEXT(material.normal_texture_index)], samplers[SAMPLER_LINEAR_CLAMP]), uv, scene_buffer.data.main_camera.mip_bias).rgb;
    vec3 local_normal = normal_sample * 2.0 - 1.0;
    vec3 normal = normalize(in_TBN * local_normal);

    float shadow = texelFetch(graph_textures[push_constants.shadow_factor_descriptor_id], ivec2(gl_FragCoord.xy), 0).r;

    vec4 occlution_roughness_metallic = texture(sampler2D(textures[nonuniformEXT(material.occlusion_roughness_metallic_texture_index)], samplers[SAMPLER_LINEAR_CLAMP]), uv, scene_buffer.data.main_camera.mip_bias);
    float ambient_occlusion = occlution_roughness_metallic.r;
    float roughness = max(occlution_roughness_metallic.g * material.roughness_factor, MIN_ROUGHNESS);
    float metallic  = occlution_roughness_metallic.b * material.metallic_factor;

    vec4 albedo = texture(sampler2D(textures[nonuniformEXT(material.color_texture_index)], samplers[SAMPLER_LINEAR_CLAMP]), uv, scene_buffer.data.main_camera.mip_bias) * material.base_color_factor;

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
    vec3 radiance = scene_buffer.data.light_color * scene_buffer.data.light_intensity;
    float sun_above_horizon = smoothstep(-0.05, 0.02, -scene_buffer.data.light_direction.y);
    vec3 Lo = (kD * albedo.rgb / 3.14159 + specular) * radiance * NdotL * shadow * sun_above_horizon;

    float gtao = 1.0;
    if (push_constants.gtao_enabled == 1u) {
        vec2 gtao_uv = gl_FragCoord.xy / (2.0 * vec2(textureSize(graph_textures[push_constants.gtao_descriptor_id], 0)));
        gtao = texture(sampler2D(graph_textures[push_constants.gtao_descriptor_id], samplers[SAMPLER_LINEAR_CLAMP]), gtao_uv).r;
    }

    vec3 F_ibl = fresnel_schlick_roughness(NdotV, F0, roughness);
    vec3 kD_ibl = (vec3(1.0) - F_ibl) * (1.0 - metallic);

    vec3 irradiance = ibl_diffuse(push_constants.sh_descriptor_id, normal);
    vec3 diffuse_ambient = kD_ibl * irradiance * albedo.rgb;

    vec3 sun_direction = normalize(-scene_buffer.data.light_direction);
    vec3 specular_ibl = ibl_specular(
        push_constants.brdf_lut_descriptor_id,
        push_constants.sh_descriptor_id,
        normal,
        V,
        roughness,
        F0,
        sun_direction,
        scene_buffer.data.time
    );
    float spec_ao = specular_occlusion(NdotV, ambient_occlusion * gtao, roughness);

    vec3 ambient = (diffuse_ambient * ambient_occlusion * gtao + specular_ibl * spec_ao) * scene_buffer.data.ibl_intensity;
    vec3 color = ambient + Lo;

    out_color = vec4(color, albedo.a);
    out_entity_index = draw_data.entity_index;
}
