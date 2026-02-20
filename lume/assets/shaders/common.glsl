#ifndef COMMON_GLSL
#define COMMON_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

#define SHAPE_BOX 0

struct SceneGpuData {
    mat4 projection_matrix;

    uint64_t indirect_buffer_device_address;
    uint64_t collider_indirect_buffer_device_address;
    uint64_t draw_count_buffer_device_address;

    uint64_t index_buffer_device_address;
    uint64_t vertex_buffer_device_address;

    uint64_t ui_index_buffer_device_address;
    uint64_t ui_vertex_buffer_device_address;

    uint64_t entity_buffer_device_address;

    uint64_t collider_buffer_device_address;

    uint64_t model_buffer_device_address;

    uint64_t material_buffer_device_address;

    uint64_t primitive_buffer_device_address;

    uint64_t draw_buffer_device_address;
};

layout(buffer_reference, std430) readonly buffer SceneBuffer  {
    SceneGpuData data;
};

layout(buffer_reference, std430) writeonly buffer IndirectBuffer  {
    uint commands[];
};

layout(buffer_reference, std430) buffer DrawCountBuffer  {
    uint entity_count;
    uint collider_count;
    uint _pad0[2];
};

struct EntityGpuData {
    mat4 transform_matrix;
    uint model_index;
    float _pad0[3];
};

layout(buffer_reference, std430) readonly buffer EntityBuffer {
    EntityGpuData data[];
};

struct ColliderGpuData {
    mat4 transform_matrix;
    vec4 half_extents;
    vec4 color;
    uint shape_type;
    float _pad0[3];
};

layout(buffer_reference, std430) readonly buffer ColliderBuffer {
    ColliderGpuData data[];
};

struct ModelGpuData {
    uint primitive_offset;
    uint primitive_count;
    uint _pad0[2];
};

layout(buffer_reference, std430) readonly buffer ModelBuffer {
    ModelGpuData data[];
};

struct MaterialGpuData {
    vec4 base_color;

    uint base_color_texture_index;
    uint _pad0[3];
};

layout(buffer_reference, std430) readonly buffer MaterialBuffer {
    MaterialGpuData data[];
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
    float position[3];
    float _pad0;
    float normal[3];
    float _pad1;
    float uv[2];
    float _pad2[2];
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
    VertexGpuData data[];
};

struct UiVertexGpuData {
    vec2 position;
    vec2 texcoord;
    vec4 color;
};

layout(buffer_reference, std430) readonly buffer UiVertexBuffer {
    UiVertexGpuData data[];
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

struct GpuRenderStatsGpuData {
    uint64_t queries[2];

    uint submeshes_rendered;
    uint submeshes_culled;

    uint _pad0[2];
};

layout(buffer_reference, std430) buffer GpuRenderStatsWrite {
    GpuRenderStatsGpuData data;
};

#endif