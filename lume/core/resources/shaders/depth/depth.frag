#version 460

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec3 world_normal;
layout(location = 0) out vec4 out_normal;

void main() {
    out_normal = vec4(normalize(world_normal), 0.0);
}
