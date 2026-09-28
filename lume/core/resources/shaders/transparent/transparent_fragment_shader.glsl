#ifndef TRANSPARENT_FRAGMENT_SHADER_GLSL
#define TRANSPARENT_FRAGMENT_SHADER_GLSL

#include "../common.glsl"

struct TransparentFragmentShader {
    uint64_t scene_buffer_device_address;
    uint64_t material_buffer_device_address;

    uint sh_descriptor_id;
    uint brdf_lut_descriptor_id;
};

#endif
