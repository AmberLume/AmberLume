#ifndef TRANSPARENT_FRAGMENT_PUSH_CONSTANTS_GLSL
#define TRANSPARENT_FRAGMENT_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"
#include "../../vertex/mesh_surface/push_constants.glsl"

struct TransparentFragmentShader {
    uint64_t scene_buffer_device_address;
    uint64_t material_buffer_device_address;

    uint sh_descriptor_id;
    uint brdf_lut_descriptor_id;
};

layout(push_constant, std430) uniform PushConstants {
    MeshSurfaceVertexShader vertex;
    TransparentFragmentShader fragment;
} push_constants;

#endif
