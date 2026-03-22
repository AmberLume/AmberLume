#ifndef COMMON_GLSL
#define COMMON_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

#define SHAPE_BOX 0

layout(buffer_reference, std430) writeonly buffer IndirectBuffer  {
    uint commands[];
};

layout(buffer_reference, std430) buffer DrawCountBuffer  {
    uint value;
};

struct MainCameraGpuData {
    mat4 projection_matrix;

    vec3 position;
    uint _pad0;

    float near;
    float far;
    uint _pad1[2];
};

struct ShadowCascadeGpuData {
    mat4 light_space_matrix;
    mat4 screen_to_light;
    float split;
    uint _pad0[3];
};

struct SceneGpuData {
    MainCameraGpuData main_camera;

    vec3 light_direction;
    uint _pad0;

    ShadowCascadeGpuData shadow_cascades[4];
    uint shadow_cascade_count;
    uint _pad1[3];
};

layout(buffer_reference, std430) readonly buffer SceneBuffer {
    SceneGpuData data;
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
    uint submesh_offset;
    uint submesh_count;
    uint _pad0[2];
};

layout(buffer_reference, std430) readonly buffer ModelBuffer {
    ModelGpuData data[];
};

struct MaterialGpuData {
    vec4 base_color_factor;
    float roughness_factor;
    float metallic_factor;

    uint color_texture_index;
    uint normal_texture_index;
    uint occlusion_roughness_metallic_texture_index;

    uint _pad0[3];
};

layout(buffer_reference, std430) readonly buffer MaterialBuffer {
    MaterialGpuData data[];
};

struct SubmeshGpuData {
    uint index_offset;
    uint index_count;
    uint vertex_offset;
    
    uint material_index;
    
    vec4 bounds_min;
    vec4 bounds_max;
};

layout(buffer_reference, std430) readonly buffer SubmeshBuffer {
    SubmeshGpuData data[];
};

struct VertexGpuData {
    float position[3];
    float _pad0;
    float normal[3];
    float _pad1;
    float tangent[4];
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

struct DrawDataGpuData {
    uint entity_index;
    uint submesh_index;
    uint _pad0[2];
};

layout(buffer_reference, std430) buffer DrawDataBuffer {
    DrawDataGpuData data[];
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

struct CullingViewGpuData {
    vec4 frustum_planes[6];

    uint64_t indirect_buffer_device_address;
    uint64_t draw_count_buffer_device_address;
    uint64_t draw_data_buffer_device_address;

    uint _pad0[2];
};

layout(buffer_reference, std430) readonly buffer CullingViewsBuffer {
    CullingViewGpuData data[];
};

struct PhysicsDebugVertexGpuData {
    vec3 point;

    uint _pad0;

    vec4 color;
};

layout(buffer_reference, std430) readonly buffer PhysicsDebugVertexBuffer {
    PhysicsDebugVertexGpuData data[];
};

#endif