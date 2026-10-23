import "../common.slang";
import "../bindings.slang";
import "../pbr.slang";
import "../sun.slang";
import "../material.slang";
import "../readback.slang";
import "../mesh_vertex.slang";
import "../texture.slang";

struct PickedEntity {
    ReadbackHeader header;

    uint id;
};

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<DrawData, Access.Read> draw_data_buffer_device_address;
    Ptr<Entity, Access.Read> entity_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;
    Ptr<VertexPosition, Access.Read> vertex_position_buffer_device_address;
    Ptr<VertexNormalTangent, Access.Read> vertex_normal_tangent_buffer_device_address;
    Ptr<VertexUv, Access.Read> vertex_uv_buffer_device_address;

    Ptr<Scene, Access.Read> scene_buffer_device_address;
    Ptr<Material, Access.Read> material_buffer_device_address;
    Ptr<PickedEntity> picked_entity_buffer_device_address;

    uint shadow_factor_descriptor_id;
    uint shadow_enabled;
    uint shadow_colored;

    uint gtao_descriptor_id;
    uint ao_enabled;

    uint sh_descriptor_id;
    uint brdf_lut_descriptor_id;

    uint pick_x;
    uint pick_y;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

struct FragmentOutput {
    float4 color : SV_Target0;
    uint entity_index : SV_Target1;
};

[shader("vertex")]
MeshSurfaceVertexOutput main(
    uint vertex_index : SV_VulkanVertexID,
    uint base_vertex : SV_StartVertexLocation,
    uint draw_id : SV_VulkanInstanceID
) {
    return mesh_surface_vertex(
        push_constants.camera_buffer_device_address,
        push_constants.draw_data_buffer_device_address,
        push_constants.entity_buffer_device_address,
        push_constants.submesh_buffer_device_address,
        push_constants.vertex_position_buffer_device_address,
        push_constants.vertex_normal_tangent_buffer_device_address,
        push_constants.vertex_uv_buffer_device_address,
        vertex_index,
        base_vertex,
        draw_id
    );
}

[shader("fragment")]
[earlydepthstencil]
FragmentOutput main(MeshSurfaceVertexOutput input) {
    Ptr<Scene, Access.Read> scene = push_constants.scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;
    DrawData draw_data = push_constants.draw_data_buffer_device_address[input.draw_id];
    Submesh submesh = push_constants.submesh_buffer_device_address[draw_data.submesh_index];
    Material material = push_constants.material_buffer_device_address[submesh.material_index];

    MaterialSurface surface = sample_material_surface(material, input.uv, input.tbn, camera.mip_bias);

    float3 shadow = float3(1.0);
    if (push_constants.shadow_enabled == 1) {
        float4 shadow_sample = graph_textures[push_constants.shadow_factor_descriptor_id].Load(int3(int2(input.position.xy), 0));
        shadow = push_constants.shadow_colored == 1 ? shadow_sample.rgb : shadow_sample.rrr;
    }

    float gtao = 1.0;
    if (push_constants.ao_enabled == 1) {
        Texture2D gtao_texture = graph_textures[push_constants.gtao_descriptor_id];

        gtao = gtao_texture.Sample(samplers[SAMPLER_LINEAR_CLAMP], input.position.xy / float2(texture_size(gtao_texture))).r;
    }

    float3 view_direction = normalize(camera.position - input.world_position);
    float3 sun_direction = normalize(-scene.light_direction);
    float n_dot_v = max(dot(surface.normal, view_direction), 0.0);
    float n_dot_l = max(dot(surface.normal, sun_direction), 0.0);

    PbrResponse direct = pbr_direct(surface.normal, view_direction, sun_direction, surface.albedo.rgb, surface.roughness, surface.metallic);
    PbrResponse ambient = pbr_ambient(
        push_constants.brdf_lut_descriptor_id,
        push_constants.sh_descriptor_id,
        surface.normal,
        view_direction,
        surface.albedo.rgb,
        surface.roughness,
        surface.metallic,
        sun_direction,
        scene.time
    );

    float ambient_occlusion = surface.ambient_occlusion * gtao;
    float specular_ambient_occlusion = specular_occlusion(n_dot_v, ambient_occlusion, surface.roughness);

    float3 direct_light = (direct.diffuse + direct.specular) * sun_irradiance(scene, n_dot_l) * shadow;
    float3 ambient_light = (ambient.diffuse * ambient_occlusion + ambient.specular * specular_ambient_occlusion) * scene.ibl_intensity;

    FragmentOutput output;
    output.color = float4(direct_light + ambient_light, surface.albedo.a);
    output.entity_index = draw_data.entity_index;

    if (uint(input.position.x) == push_constants.pick_x && uint(input.position.y) == push_constants.pick_y) {
        Ptr<PickedEntity> picked_entity = push_constants.picked_entity_buffer_device_address;

        InterlockedOr(picked_entity.header.written, 1);

        picked_entity.id = draw_data.entity_index;
    }

    return output;
}
