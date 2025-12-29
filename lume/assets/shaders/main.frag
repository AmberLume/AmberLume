#version 460
#extension GL_ARB_shader_draw_parameters : enable

#include "common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in flat uint draw_id;
layout(location = 0) out vec4 out_color;

void main() {
    SceneGpuData scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    DrawBufferRead draws = DrawBufferRead(scene.draw_buffer_device_address);
    DrawGpuData draw = draws.data[draw_id];

    PrimitiveBuffer primitives = PrimitiveBuffer(scene.primitive_buffer_device_address);
    PrimitiveGpuData primitive = primitives.data[draw.primitive_index];

    MaterialBuffer materials = MaterialBuffer(scene.material_buffer_device_address);
    vec4 base_color;
    if (is_material_available(scene, primitive.material_index)) {
        base_color = materials.data[primitive.material_index].base_color;
    } else {
        base_color = vec4(0.0, 0.0, 0.0, 1.0);
    }

    vec3 normal = normalize(fragNormal);

    vec3 keyLight = normalize(vec3(0.5, 1.0, 0.5));
    vec3 fillLight = normalize(vec3(-0.3, 0.2, 0.8));

    float keyDiffuse = max(dot(normal, keyLight), 0.0);
    float fillDiffuse = max(dot(normal, fillLight), 0.0);

    float ambient = 0.3;
    float lighting = ambient + keyDiffuse * 0.5 + fillDiffuse * 0.3;

    out_color = vec4(vec3(base_color) * lighting, 1.0);
}
