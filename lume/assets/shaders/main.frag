#version 460
#extension GL_ARB_shader_draw_parameters : enable
#extension GL_EXT_nonuniform_qualifier : enable

#include "common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

layout(set = 0, binding = 0) uniform sampler2D textures[];

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec2 uv;
layout(location = 2) in flat uint draw_id;
layout(location = 0) out vec4 out_color;

void main() {
    SceneGpuData scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    DrawBufferRead draws = DrawBufferRead(scene.draw_buffer_device_address);
    DrawGpuData draw = draws.data[draw_id];

    PrimitiveBuffer primitives = PrimitiveBuffer(scene.primitive_buffer_device_address);
    PrimitiveGpuData primitive = primitives.data[draw.primitive_index];

    MaterialBuffer materials = MaterialBuffer(scene.material_buffer_device_address);

    out_color = vec4(0.0, 0.0, 0.0, 1.0);

    if (is_resource_available(scene.material_availability_buffer_device_address, primitive.material_index)) {
        MaterialGpuData material = materials.data[primitive.material_index];

        vec3 normal = normalize(frag_normal);

        vec3 key_light = normalize(vec3(0.5, 1.0, 0.5));
        vec3 fill_light = normalize(vec3(-0.3, 0.2, 0.8));

        float key_diffuse = max(dot(normal, key_light), 0.0);
        float fill_diffuse = max(dot(normal, fill_light), 0.0);

        float ambient = 0.3;
        float lighting = ambient + key_diffuse * 0.5 + fill_diffuse * 0.3;

        if (is_resource_available(scene.image_availability_buffer_device_address, material.base_color_texture_index)) {
            vec4 texture = texture(textures[nonuniformEXT(material.base_color_texture_index)], uv);

            out_color = vec4(texture.rgb * lighting, 1.0);
        } else {
            out_color = material.base_color * lighting;
        }
    }
}
