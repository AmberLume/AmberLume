#ifndef PHYSICS_DEBUG_VERTEX_PUSH_CONSTANTS_GLSL
#define PHYSICS_DEBUG_VERTEX_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct PhysicsDebugVertexShader {
    uint64_t camera_buffer_device_address;
    uint64_t physics_debug_vertex_buffer_device_address;
};

#endif
