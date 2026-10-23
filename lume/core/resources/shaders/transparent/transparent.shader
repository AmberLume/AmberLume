import "../common.slang";
import "../pbr.slang";
import "../sun.slang";
import "../material.slang";
import "../mesh_vertex.slang";

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

    uint sh_descriptor_id;
    uint brdf_lut_descriptor_id;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

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
float4 main(MeshSurfaceVertexOutput input) : SV_Target0 {
    Ptr<Scene, Access.Read> scene = push_constants.scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;
    DrawData draw_data = push_constants.draw_data_buffer_device_address[input.draw_id];
    Submesh submesh = push_constants.submesh_buffer_device_address[draw_data.submesh_index];
    Material material = push_constants.material_buffer_device_address[submesh.material_index];

    MaterialSurface surface = sample_material_surface(material, input.uv, input.tbn, camera.mip_bias);

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

    float specular_ambient_occlusion = specular_occlusion(n_dot_v, surface.ambient_occlusion, surface.roughness);
    float3 sun_light = sun_irradiance(scene, n_dot_l);

    float3 diffuse = direct.diffuse * sun_light + ambient.diffuse * surface.ambient_occlusion * scene.ibl_intensity;
    float3 specular = direct.specular * sun_light + ambient.specular * specular_ambient_occlusion * scene.ibl_intensity;

    float reflectance = pbr_reflectance(surface.normal, view_direction, surface.albedo.rgb, surface.roughness, surface.metallic);
    float opacity = surface.albedo.a + (1.0 - surface.albedo.a) * reflectance;

    return float4(diffuse * surface.albedo.a + specular, opacity);
}
