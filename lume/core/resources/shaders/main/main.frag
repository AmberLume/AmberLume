#version 460
#extension GL_ARB_shader_draw_parameters : enable
#extension GL_EXT_nonuniform_qualifier : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform sampler2D textures[];
layout(set = 0, binding = 1) uniform sampler2DArrayShadow shadow_arrays[];

layout(location = 0) in mat3 in_TBN;
layout(location = 3) in vec2 uv;
layout(location = 4) in flat uint draw_id;
layout(location = 5) in vec3 world_pos;

layout(location = 0) out vec4 out_color;

float compute_shadow(vec3 world_pos_in, SceneBuffer scene_buffer) {
    ShadowCascadesBuffer cascades = ShadowCascadesBuffer(push_constants.shadow_cascades_buffer_device_address);

    vec4 view_pos = scene_buffer.data.main_camera.view_projection * vec4(world_pos_in, 1.0);
    vec3 ndc = view_pos.xyz / view_pos.w;
    float depth = ndc.z;

    float near = scene_buffer.data.main_camera.near;
    float far = scene_buffer.data.main_camera.far;
    float view_z = (near * far) / (far - depth * (far - near));

    uint cascade_index = 0;
    for (uint i = 0; i < scene_buffer.data.shadow_cascade_count - 1; ++i) {
        if (view_z > cascades.data[i].split) {
            cascade_index = i + 1;
        }
    }

    vec4 light_clip = cascades.data[cascade_index].light_space_matrix * vec4(world_pos_in, 1.0);
    vec3 light_ndc = light_clip.xyz / light_clip.w;
    vec2 shadow_uv = light_ndc.xy * 0.5 + 0.5;
    float receiver_z = light_ndc.z - push_constants.shadow_bias;

    if (any(lessThan(shadow_uv, vec2(0.0))) || any(greaterThan(shadow_uv, vec2(1.0)))) {
        return 1.0;
    }

    int radius = push_constants.shadow_pcf_radius;
    float texel_size = 1.0 / float(textureSize(shadow_arrays[push_constants.shadow_array_descriptor_id], 0).x);

    float sum = 0.0;
    float count = 0.0;
    for (int x = -radius; x <= radius; x++) {
        for (int y = -radius; y <= radius; y++) {
            vec2 offset = vec2(float(x), float(y)) * texel_size;
            sum += texture(
                shadow_arrays[push_constants.shadow_array_descriptor_id],
                vec4(shadow_uv + offset, float(cascade_index), receiver_z)
            );
            count += 1.0;
        }
    }
    return sum / count;
}

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];
    Material material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec3 normal_sample = texture(textures[nonuniformEXT(material.normal_texture_index)], uv).rgb;
    vec3 local_normal = normal_sample * 2.0 - 1.0;
    vec3 normal = normalize(in_TBN * local_normal);

    float shadow = compute_shadow(world_pos, scene_buffer);

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
