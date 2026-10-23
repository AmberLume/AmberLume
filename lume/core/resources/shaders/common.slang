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

struct VertexPosition {
    float position[3];
};

struct VertexNormalTangent {
    float normal[3];
    float tangent[4];
};

struct VertexUv {
    float uv[2];
};

struct DrawData {
    uint entity_index;
    uint submesh_index;
    int previous_vertex_delta;
    uint cascade_mask;
    float sort_key;
};
