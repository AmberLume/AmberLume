#ifndef RT_TRANSMISSIVE_SHADOW_PUSH_CONSTANTS_GLSL
#define RT_TRANSMISSIVE_SHADOW_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    vec4 sun_direction;

    uint64_t scene_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t mesh_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t material_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint transmittance_storage_id;
    uint tlas_descriptor_id;

    float sun_angular_radius;
    uint sample_count;
    uint frame_number;
} push_constants;

#endif
