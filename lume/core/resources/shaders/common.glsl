#ifndef COMMON_GLSL
#define COMMON_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

#define SHAPE_BOX 0

layout(buffer_reference, std430) writeonly buffer IndirectBuffer  {
    uint commands[];
};

layout(buffer_reference, std430) readonly buffer IndirectReadBuffer  {
    uint commands[];
};

layout(buffer_reference, std430) buffer DrawCountBuffer  {
    uint values[];
};

struct MainCamera {
    mat4 view_projection;
    mat4 previous_view_projection;
    mat4 jittered_view_projection;
    mat4 inverse_view_projection;
    mat4 inverse_jittered_view_projection;

    mat4 view;

    vec3 position;
    uint _pad0;

    float near;
    float far;
    vec2 ndc_to_view_mul;
    vec2 ndc_to_view_add;
    float mip_bias;
    uint _pad1;
};

struct ShadowCascade {
    mat4 light_space_matrix;
    float split;
    float world_radius;
    uint _pad0[2];
};

layout(buffer_reference, std430) readonly buffer ShadowCascadesBuffer {
    ShadowCascade data[];
};

struct Scene {
    MainCamera main_camera;

    vec3 light_direction;
    float light_intensity;

    vec3 light_color;
    float ibl_intensity;

    uint shadow_cascade_count;
    float time;
    uint _pad1[2];
};

layout(buffer_reference, std430) readonly buffer SceneBuffer {
    Scene data;
};

struct Entity {
    mat4 transform_matrix;
    mat4 previous_transform_matrix;
    vec4 outline;
    uint mesh_index;
    uint is_skinned;
    uint _pad0;
    uint bone_transform_offset;
};

layout(buffer_reference, std430) readonly buffer EntityBuffer {
    Entity data[];
};

struct Mesh {
    uint submesh_offset;
    uint submesh_count;
    uint _pad0[2];
};

layout(buffer_reference, std430) readonly buffer MeshBuffer {
    Mesh data[];
};

const uint MATERIAL_FLAG_ALPHA_OPAQUE = 1u;
const uint MATERIAL_FLAG_ALPHA_MASK = 2u;
const uint MATERIAL_FLAG_ALPHA_BLEND = 4u;

const uint MATERIAL_ALPHA_MODE_BITS = MATERIAL_FLAG_ALPHA_OPAQUE | MATERIAL_FLAG_ALPHA_MASK | MATERIAL_FLAG_ALPHA_BLEND;

const uint RT_INSTANCE_MASK_OPAQUE = 1u;
const uint RT_INSTANCE_MASK_BLEND = 2u;

struct Material {
    vec4 base_color_factor;
    float roughness_factor;
    float metallic_factor;

    uint color_texture_index;
    uint normal_texture_index;
    uint occlusion_roughness_metallic_texture_index;

    uint flags;
    float alpha_cutoff;

    uint _pad0;
};

layout(buffer_reference, std430) readonly buffer MaterialBuffer {
    Material data[];
};

struct Submesh {
    uint index_offset;
    uint index_count;
    uint vertex_offset;
    
    uint material_index;
    
    vec4 bounds_min;
    vec4 bounds_max;
};

layout(buffer_reference, std430) readonly buffer SubmeshBuffer {
    Submesh data[];
};

struct Vertex {
    float position[3];
    float _pad0;
    float normal[3];
    float _pad1;
    float tangent[4];
    float uv[2];
    uint bone_indices[2];
    float bone_weights[4];
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
    Vertex data[];
};

struct UiVertex {
    vec2 position;
    vec2 texcoord;
    vec4 color;
};

layout(buffer_reference, std430) readonly buffer UiVertexBuffer {
    UiVertex data[];
};

struct DrawData {
    uint entity_index;
    uint submesh_index;
    uint cascade_mask;
    float sort_key;
};

layout(buffer_reference, std430) buffer DrawDataBuffer {
    DrawData data[];
};

struct CullRequest {
    uint accept_mask;
    uint count_index;
    uint draw_offset;
    uint capacity;
};

layout(buffer_reference, std430) readonly buffer CullRequestsBuffer {
    CullRequest data[];
};

struct CullingView {
    vec4 frustum_planes[6];
};

layout(buffer_reference, std430) readonly buffer CullingViewsBuffer {
    CullingView data[];
};

struct PhysicsDebugVertex {
    vec3 point;

    uint _pad0;

    vec4 color;
};

layout(buffer_reference, std430) readonly buffer PhysicsDebugVertexBuffer {
    PhysicsDebugVertex data[];
};

struct SkinningInstance {
    uint animation_id;
    uint skeleton_id;
    uint bone_transform_offset;
    float time;

    uint previous_animation_id;
    float previous_time;
    float blend_factor;
    
    uint _pad0;
};

layout(buffer_reference, std430) buffer SkinningInstanceBuffer {
    SkinningInstance data[];
};

struct Animation {
    uint offset;
    uint bone_count;
    uint frame_count;
    float duration;
    float fps;
    uint _pad0[3];
};

layout(buffer_reference, std430) buffer AnimationBuffer {
    Animation data[];
};

struct AnimationFrame {
    vec3 translation;
    uint _pad0;
    vec4 rotation;
    vec3 scale;
    uint _pad1;
};

layout(buffer_reference, std430) buffer AnimationFrameBuffer {
    AnimationFrame data[];
};

struct Skeleton {
    uint bone_offset;
    uint bone_count;
    uint _pad[2];
};

layout(buffer_reference, std430) buffer SkeletonBuffer {
    Skeleton data[];
};

struct SkeletonBone {
    int parent_index;
    uint _pad[3];
    mat4 inverse_bind_matrix;
};

layout(buffer_reference, std430) buffer SkeletonBoneBuffer {
    SkeletonBone data[];
};

struct BoneTransform {
    mat4 transform;
};

layout(buffer_reference, std430) buffer BoneTransformBuffer {
    BoneTransform data[];
};

#endif