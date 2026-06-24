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

const float VOGEL_GOLDEN_ANGLE = 2.39996323;
const float MIN_ROUGHNESS = 0.045;

float sample_cascade(
    uint cascade_index,
    vec3 shadow_pos,
    ShadowCascadesBuffer cascades,
    float angle,
    float radial_offset,
    vec3 dpdx,
    vec3 dpdy
) {
    mat4 light_matrix = cascades.data[cascade_index].light_space_matrix;
    vec4 light_clip = light_matrix * vec4(shadow_pos, 1.0);
    vec3 light_ndc = light_clip.xyz / light_clip.w;
    vec2 shadow_uv = light_ndc.xy * 0.5 + 0.5;
    float receiver_z = light_ndc.z;

    if (any(lessThan(shadow_uv, vec2(0.0))) || any(greaterThan(shadow_uv, vec2(1.0)))) {
        return 1.0;
    }

    int sample_count = int(min(push_constants.shadow_pcf_sample_count, 16u));
    if (push_constants.shadow_pcf_world_radius <= 0.0 || sample_count <= 1) {
        return texture(
            sampler2DArrayShadow(shadow_arrays[push_constants.shadow_array_descriptor_id], samplers[SAMPLER_SHADOW]),
            vec4(shadow_uv, float(cascade_index), receiver_z)
        );
    }

    vec4 dclip_dx = light_matrix * vec4(dpdx, 0.0);
    vec4 dclip_dy = light_matrix * vec4(dpdy, 0.0);
    vec2 duv_dx = dclip_dx.xy * 0.5;
    vec2 duv_dy = dclip_dy.xy * 0.5;
    float det = duv_dx.x * duv_dy.y - duv_dy.x * duv_dx.y;
    vec2 dz_duv = vec2(0.0);
    if (abs(det) > 1e-12) {
        dz_duv = vec2(
            duv_dy.y * dclip_dx.z - duv_dx.y * dclip_dy.z,
            duv_dx.x * dclip_dy.z - duv_dy.x * dclip_dx.z
        ) / det;
    }

    float uv_radius = push_constants.shadow_pcf_world_radius / (2.0 * cascades.data[cascade_index].world_radius);
    float sum = 0.0;
    for (int i = 0; i < sample_count; i++) {
        float r = sqrt((float(i) + radial_offset) / float(sample_count));
        float theta = float(i) * VOGEL_GOLDEN_ANGLE + angle;
        vec2 offset = r * vec2(cos(theta), sin(theta)) * uv_radius;
        float sample_z = receiver_z + dot(offset, dz_duv);
        sum += texture(
            sampler2DArrayShadow(shadow_arrays[push_constants.shadow_array_descriptor_id], samplers[SAMPLER_SHADOW]),
            vec4(shadow_uv + offset, float(cascade_index), sample_z)
        );
    }
    return sum / float(sample_count);
}

float compute_shadow(vec3 world_pos_in, vec3 geom_normal, SceneBuffer scene_buffer) {
    ShadowCascadesBuffer cascades = ShadowCascadesBuffer(push_constants.shadow_cascades_buffer_device_address);

    vec3 dpdx = dFdx(world_pos_in);
    vec3 dpdy = dFdy(world_pos_in);

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
    float angle = fract(ign + float(push_constants.frame_index) * 0.61803398875) * 6.28318530718;
    float radial_offset = fract(ign + float(push_constants.frame_index) * 0.41421356237);

    float shadow_curr = sample_cascade(cascade_index, shadow_pos, cascades, angle, radial_offset, dpdx, dpdy);

    float blend_range = push_constants.shadow_cascade_blend_range;
    if (blend_range > 0.0 && cascade_index + 1 < cascade_count) {
        float curr_split = cascades.data[cascade_index].split;
        float prev_split = cascade_index > 0 ? cascades.data[cascade_index - 1].split : near;
        float fade_start = curr_split - (curr_split - prev_split) * blend_range;
        float fade = smoothstep(fade_start, curr_split, view_z);
        if (fade > 0.0) {
            float shadow_next = sample_cascade(cascade_index + 1, shadow_pos, cascades, angle, radial_offset, dpdx, dpdy);
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

    vec3 normal_sample = texture(sampler2D(textures[nonuniformEXT(material.normal_texture_index)], samplers[SAMPLER_LINEAR_CLAMP]), uv, scene_buffer.data.main_camera.mip_bias).rgb;
    vec3 local_normal = normal_sample * 2.0 - 1.0;
    vec3 normal = normalize(in_TBN * local_normal);
    vec3 geom_normal = normalize(in_TBN[2]);

    float shadow = compute_shadow(world_pos, geom_normal, scene_buffer);

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
