#ifndef MAIN_FRAGMENT_PUSH_CONSTANTS_GLSL
#define MAIN_FRAGMENT_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"
#include "../../vertex/mesh_surface/push_constants.glsl"

struct MainFragmentShader {
    uint64_t scene_buffer_device_address;
    uint64_t material_buffer_device_address;
    uint64_t picked_entity_buffer_device_address;

    uint shadow_factor_descriptor_id;
    uint shadow_enabled;
    uint shadow_colored;

    uint gtao_descriptor_id;
    uint ao_enabled;

    uint sh_descriptor_id;
    uint brdf_lut_descriptor_id;

    uint pick_x;
    uint pick_y;

    uint _pad0;
};

layout(push_constant, std430) uniform PushConstants {
    MeshSurfaceVertexShader vertex;
    MainFragmentShader fragment;
} push_constants;

#endif
