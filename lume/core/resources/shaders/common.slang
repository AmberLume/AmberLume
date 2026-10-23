static const uint MATERIAL_FLAG_ALPHA_OPAQUE = 1u;
static const uint MATERIAL_FLAG_ALPHA_MASK = 2u;
static const uint MATERIAL_FLAG_ALPHA_BLEND = 4u;

static const uint MATERIAL_ALPHA_MODE_BITS = MATERIAL_FLAG_ALPHA_OPAQUE | MATERIAL_FLAG_ALPHA_MASK | MATERIAL_FLAG_ALPHA_BLEND;

static const uint RT_INSTANCE_MASK_OPAQUE = 1u;
static const uint RT_INSTANCE_MASK_BLEND = 2u;

struct Camera {
    float4x4 view_projection;
    float4x4 previous_view_projection;
    float4x4 inverse_view_projection;

    float4x4 view;

    float3 position;
    uint _pad0;

    float2 tan_half_fov;
    float near;
    float far;

    float2 jitter;
    float mip_bias;
    uint _pad1;
};

struct ShadowCascade {
    float4x4 view_projection;
    float split;
    float world_radius;
    uint _pad0[2];
};

struct Scene {
    float3 light_direction;
    float light_intensity;

    float3 light_color;
    float ibl_intensity;

    uint shadow_cascade_count;
    float time;
    uint _pad0[2];
};

struct Entity {
    float4x4 transform_matrix;
    float4x4 previous_transform_matrix;
    uint mesh_index;
    uint _pad0[3];
};

struct EntityOutline {
    float4 outline;
};

struct Mesh {
    uint submesh_offset;
    uint submesh_count;
    uint binding_offset;
    uint vertex_slice_stride;
    uint vertex_slice_count;
};

struct Material {
    float4 base_color_factor;
    float roughness_factor;
    float metallic_factor;

    uint color_texture_index;
    uint normal_texture_index;
    uint occlusion_roughness_metallic_texture_index;

    uint flags;
    float alpha_cutoff;

    uint _pad0;
};

struct Submesh {
    uint index_offset;
    uint index_count;
    uint vertex_offset;
    uint uv_offset;
    uint material_index;
    uint bounds_index;
};

struct SubmeshBounds {
    float3 min;
    float3 max;
};

struct VertexPosition {
    float3 position;
};

struct VertexNormalTangent {
    float3 normal;
    float4 tangent;
};

struct VertexUv {
    float2 uv;
};

struct DrawData {
    uint entity_index;
    uint submesh_index;
    int previous_vertex_delta;
    uint cascade_mask;
    float sort_key;
};

struct DrawIndexedIndirectCommand {
    uint index_count;
    uint instance_count;
    uint first_index;
    int vertex_offset;
    uint first_instance;
};

struct CullingView {
    float4 frustum_planes[6];
};

struct SkinningPose {
    uint animation_id;
    float time;

    uint blend_from_animation_id;
    float blend_from_time;
    float blend_factor;
};

struct SkinningInstance {
    uint mesh_id;
    uint skeleton_id;
    uint bone_transform_offset;
    uint bounds_index;

    SkinningPose pose;
};

struct BoneTransform {
    float4x4 transform;
};

struct DepthReduceResult {
    uint farthest_depth_bits;
};
