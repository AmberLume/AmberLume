#version 460
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

    uint64_t primitive_buffer_device_address;
};

struct EntityGpuData {
    mat4 transform_matrix;
    uint model_index;
    float _pad0[3];
};

struct VertexGpuData {
    vec3 position;
    float _pad0;
    vec3 normal;
    float _pad1;
    vec2 uv;
    vec2 _pad2;
};

layout(buffer_reference, std430) readonly buffer SceneBuffer  {
    SceneGpuData data;
};

layout(buffer_reference, std430) readonly buffer EntityBuffer {
    EntityGpuData data[];
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
    VertexGpuData data[];
};

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

void main() {
    SceneGpuData scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    EntityBuffer entities = EntityBuffer(scene.entity_buffer_device_address);
    EntityGpuData entity = entities.data[gl_InstanceIndex];

    VertexBuffer vertices = VertexBuffer(scene.vertex_buffer_device_address);
    VertexGpuData vertex = vertices.data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position, 1.0);

    gl_Position = scene.projection_matrix * world_position;
}
