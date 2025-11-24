#version 460
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

struct Material {
    uint albedo_index;
    uint normal_index;
    uint metallic_roughness_index;
    uint _padding;
};

layout(buffer_reference, scalar) buffer MaterialBuffer {
    Material data[];
};

layout(set=1, binding=0) uniform texture2D textures[];
layout(set=1, binding=1) uniform sampler samplers[];

layout(push_constant) uniform PushConstants {
    uint64_t vertex_buffer_address;
    uint64_t index_buffer_address;
    uint64_t entity_buffer_address;
    uint64_t material_buffer_address;
} push_constant;

layout(location=0) in vec3 in_normal;
layout(location=1) in vec2 in_uv;
layout(location=2) flat in uint in_material_id;

layout(location=0) out vec4 out_color;

void main() {
    MaterialBuffer materials = MaterialBuffer(push_constant.material_buffer_address);

    uint albedo_idx = materials.data[in_material_id].albedo_index;

    vec4 albedo = texture(
        sampler2D(textures[albedo_idx], samplers[0]),
        in_uv
    );

    out_color = albedo;
}
