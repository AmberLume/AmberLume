#ifndef COMMON_GLSL
#define COMMON_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

struct SceneGpuData {
    mat4 projection_matrix;

    uint64_t indirect_buffer_device_address;
    uint64_t draw_count_buffer_device_address;

    uint64_t index_buffer_device_address;
    uint64_t vertex_buffer_device_address;

    uint64_t entity_buffer_device_address;

    uint64_t model_buffer_device_address;
    uint64_t model_availability_buffer_device_address;

    uint64_t material_buffer_device_address;
    uint64_t material_availability_buffer_device_address;

    uint64_t primitive_buffer_device_address;

    uint64_t draw_buffer_device_address;
};

layout(buffer_reference, std430) readonly buffer SceneBuffer  {
    SceneGpuData data;
};

layout(buffer_reference, std430) writeonly buffer IndirectBuffer  {
    uint commands[];
};

layout(buffer_reference, std430) writeonly buffer DrawCountBuffer  {
    uint count;
};

struct EntityGpuData {
    mat4 transform_matrix;
    uint model_index;
    float _pad0[3];
};

layout(buffer_reference, std430) readonly buffer EntityBuffer {
    EntityGpuData data[];
};

struct ModelGpuData {
    uint primitive_offset;
    uint primitive_count;
    uint _pad0[2];
};

layout(buffer_reference, std430) readonly buffer ModelBuffer {
    ModelGpuData data[];
};

layout(buffer_reference, std430) readonly buffer ModelAvailabilityBuffer {
    uint bits[];
};

struct MaterialGpuData {
    vec4 base_color;

    uint base_color_texture_index;
    uint _pad0[3];
};

layout(buffer_reference, std430) readonly buffer MaterialBuffer {
    MaterialGpuData data[];
};

layout(buffer_reference, std430) readonly buffer MaterialAvailabilityBuffer {
    uint bits[];
};

struct PrimitiveGpuData {
    uint index_offset;
    uint index_count;
    uint vertex_offset;

    uint material_index;
};

layout(buffer_reference, std430) readonly buffer PrimitiveBuffer {
    PrimitiveGpuData data[];
};

struct VertexGpuData {
    vec3 position;
    float _pad0;
    vec3 normal;
    float _pad1;
    vec2 uv;
    vec2 _pad2;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
    VertexGpuData data[];
};

struct DrawGpuData {
    uint entity_index;
    uint primitive_index;
    uint _pad0[2];
};

layout(buffer_reference, std430) writeonly buffer DrawBufferWrite {
    DrawGpuData data[];
};

layout(buffer_reference, std430) readonly buffer DrawBufferRead {
    DrawGpuData data[];
};

bool is_model_available(
    SceneGpuData scene,
    uint model_index
) {
    ModelAvailabilityBuffer model_availability_buffer = ModelAvailabilityBuffer(scene.model_availability_buffer_device_address);

    uint word = model_index / 32;
    uint bit = model_index % 32;

    return (model_availability_buffer.bits[word] & (1u << bit)) != 0;
}

bool is_material_available(
    SceneGpuData scene,
    uint material_index
) {
    MaterialAvailabilityBuffer material_availability_buffer = MaterialAvailabilityBuffer(scene.material_availability_buffer_device_address);

    uint word = material_index / 32;
    uint bit = material_index % 32;

    return (material_availability_buffer.bits[word] & (1u << bit)) != 0;
}

#endif