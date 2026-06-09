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
layout(location = 1) out uint out_entity_index;

const vec2 POISSON_DISK_16[16] = vec2[16](
    vec2(-0.94201624, -0.39906216),
    vec2( 0.94558609, -0.76890725),
    vec2(-0.09418410, -0.92938870),
    vec2( 0.34495938,  0.29387760),
    vec2(-0.91588581,  0.45771432),
    vec2(-0.81544232, -0.87912464),
    vec2(-0.38277543,  0.27676845),
    vec2( 0.97484398,  0.75648379),
    vec2( 0.44323325, -0.97511554),
    vec2( 0.53742981, -0.47373420),
    vec2(-0.26496911, -0.41893023),
    vec2( 0.79197514,  0.19090188),
    vec2(-0.24188840,  0.99706507),
    vec2(-0.81409955,  0.91437590),
    vec2( 0.19984126,  0.78641367),
    vec2( 0.14383161, -0.14100790)
);

float sample_cascade(
    uint cascade_index,
    vec3 shadow_pos,
    ShadowCascadesBuffer cascades,
    mat2 rot
) {
    vec4 light_clip = cascades.data[cascade_index].light_space_matrix * vec4(shadow_pos, 1.0);
    vec3 light_ndc = light_clip.xyz / light_clip.w;
    vec2 shadow_uv = light_ndc.xy * 0.5 + 0.5;
    float receiver_z = light_ndc.z;

    if (any(lessThan(shadow_uv, vec2(0.0))) || any(greaterThan(shadow_uv, vec2(1.0)))) {
        return 1.0;
    }

    int sample_count = int(min(push_constants.shadow_pcf_sample_count, 16u));
    if (push_constants.shadow_pcf_world_radius <= 0.0 || sample_count <= 1) {
        return texture(
            shadow_arrays[push_constants.shadow_array_descriptor_id],
            vec4(shadow_uv, float(cascade_index), receiver_z)
        );
    }

    float uv_radius = push_constants.shadow_pcf_world_radius / (2.0 * cascades.data[cascade_index].world_radius);
    float sum = 0.0;
    for (int i = 0; i < sample_count; i++) {
        vec2 offset = rot * POISSON_DISK_16[i] * uv_radius;
        sum += texture(
            shadow_arrays[push_constants.shadow_array_descriptor_id],
            vec4(shadow_uv + offset, float(cascade_index), receiver_z)
        );
    }
    return sum / float(sample_count);
}

float compute_shadow(vec3 world_pos_in, vec3 geom_normal, SceneBuffer scene_buffer) {
    ShadowCascadesBuffer cascades = ShadowCascadesBuffer(push_constants.shadow_cascades_buffer_device_address);

    vec4 view_pos = scene_buffer.data.main_camera.view_projection * vec4(world_pos_in, 1.0);
    vec3 ndc = view_pos.xyz / view_pos.w;
    float depth = ndc.z;

    float near = scene_buffer.data.main_camera.near;
    float far = scene_buffer.data.main_camera.far;
    float view_z = (near * far) / (far - depth * (far - near));

    uint cascade_count = scene_buffer.data.shadow_cascade_count;
    uint cascade_index = 0;
    for (uint i = 0; i < cascade_count - 1; ++i) {
        if (view_z > cascades.data[i].split) {
            cascade_index = i + 1;
        }
    }

    vec3 light_dir = normalize(-scene_buffer.data.light_direction);
    float n_dot_l = clamp(dot(geom_normal, light_dir), 0.0, 1.0);

    vec3 shadow_pos = world_pos_in
        + geom_normal * push_constants.shadow_normal_bias * (1.0 - n_dot_l)
        + light_dir * push_constants.shadow_bias;

    float ign = fract(52.9829189 * fract(0.06711056 * gl_FragCoord.x + 0.00583715 * gl_FragCoord.y));
    float phi = ign * 6.28318530718;
    float c = cos(phi);
    float s = sin(phi);
    mat2 rot = mat2(c, -s, s, c);

    float shadow_curr = sample_cascade(cascade_index, shadow_pos, cascades, rot);

    float blend_range = push_constants.shadow_cascade_blend_range;
    if (blend_range > 0.0 && cascade_index + 1 < cascade_count) {
        float curr_split = cascades.data[cascade_index].split;
        float prev_split = cascade_index > 0 ? cascades.data[cascade_index - 1].split : near;
        float fade_start = curr_split - (curr_split - prev_split) * blend_range;
        float fade = smoothstep(fade_start, curr_split, view_z);
        if (fade > 0.0) {
            float shadow_next = sample_cascade(cascade_index + 1, shadow_pos, cascades, rot);
            return mix(shadow_curr, shadow_next, fade);
        }
    }
    return shadow_curr;
}

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];
    Material material = MaterialBuffer(push_constants.material_buffer_device_address).data[submesh.material_index];

    vec3 normal_sample = texture(textures[nonuniformEXT(material.normal_texture_index)], uv).rgb;
    vec3 local_normal = normal_sample * 2.0 - 1.0;
    vec3 normal = normalize(in_TBN * local_normal);
    vec3 geom_normal = normalize(in_TBN[2]);

    float shadow = compute_shadow(world_pos, geom_normal, scene_buffer);

    vec4 occlution_roughness_metallic = texture(textures[nonuniformEXT(material.occlusion_roughness_metallic_texture_index)], uv);
    float ambient_occlusion = occlution_roughness_metallic.r;
    float roughness = occlution_roughness_metallic.g * material.roughness_factor;
    float metallic  = occlution_roughness_metallic.b * material.metallic_factor;

    vec4 albedo = texture(textures[nonuniformEXT(material.color_texture_index)], uv) * material.base_color_factor;

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

    out_color = vec4(color, albedo.a);
    out_entity_index = draw_data.entity_index;
}
