#version 460

#include "../common.glsl"
#include "push_constants.glsl"

struct TerrainChunkView {
    float center[3];
    uint level;
    uint vertex_offset;

    uint _pad0[3];
};

layout(buffer_reference, std430) readonly buffer TerrainChunkViewBuffer {
    TerrainChunkView data[];
};

layout(location = 0) out vec4 v_color;

vec3 level_color(uint level) {
    const float intensity = 1.6;

    switch (level) {
        case 0u: return vec3(1.0, 0.15, 0.15) * intensity;
        case 1u: return vec3(1.0, 0.55, 0.05) * intensity;
        case 2u: return vec3(1.0, 1.0, 0.15) * intensity;
        case 3u: return vec3(0.2, 1.0, 0.25) * intensity;
        case 4u: return vec3(0.2, 0.55, 1.0) * intensity;
        default: return vec3(0.7, 0.25, 1.0) * intensity;
    }
}

void main() {
    Scene scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    uint chunk_index = uint(gl_VertexIndex) / push_constants.node_count;
    uint local_index = uint(gl_VertexIndex) % push_constants.node_count;

    TerrainChunkView chunk =
        TerrainChunkViewBuffer(push_constants.chunk_buffer_device_address).data[chunk_index];

    Vertex vertex =
        VertexBuffer(push_constants.vertex_buffer_device_address).data[chunk.vertex_offset + local_index];

    vec3 center = vec3(chunk.center[0], chunk.center[1], chunk.center[2]);
    vec3 local = vec3(vertex.position[0], vertex.position[1], vertex.position[2]);

    vec3 world = center + local;

    vec3 camera_up = vec3(
        scene.main_camera.view[0][1],
        scene.main_camera.view[1][1],
        scene.main_camera.view[2][1]
    );

    vec4 clip = scene.main_camera.view_projection * vec4(world, 1.0);
    vec4 clip_offset =
        scene.main_camera.view_projection * vec4(world + camera_up * push_constants.point_size, 1.0);

    float projected = distance(clip.xy / clip.w, clip_offset.xy / clip_offset.w);

    gl_Position = clip;
    gl_PointSize = clamp(projected * push_constants.viewport_height * 0.5, 1.0, 64.0);

    v_color = vec4(level_color(chunk.level), 1.0);
}
