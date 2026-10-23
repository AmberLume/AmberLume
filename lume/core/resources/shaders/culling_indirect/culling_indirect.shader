import "../common.slang";
import "../aabb.slang";
import "../readback.slang";

struct CullRequest {
    uint accept_mask;
    uint count_index;
    uint draw_offset;
    uint capacity;
};

struct CullingIndirectMetaStatistics {
    uint submeshes_rendered;
    uint submeshes_culled;
    uint submeshes_dropped;

    uint _pad0;
};

struct CullingIndirectMetaStatisticsReadback {
    ReadbackHeader header;

    CullingIndirectMetaStatistics data[];
};

struct PushConstants {
    Ptr<CullingView, Access.Read> culling_views_buffer_device_address;
    Ptr<Entity, Access.Read> entity_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;
    Ptr<SubmeshBounds, Access.Read> submesh_bounds_buffer_device_address;
    Ptr<CullingIndirectMetaStatisticsReadback> meta_statistics_buffer_device_address;

    Ptr<CullRequest, Access.Read> cull_requests_buffer_device_address;
    Ptr<DrawIndexedIndirectCommand> indirect_buffer_device_address;
    Ptr<uint> draw_count_buffer_device_address;
    Ptr<DrawData> draw_data_buffer_device_address;
    Ptr<Material, Access.Read> material_buffer_device_address;
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint view_count;
    uint entity_count;
    uint combine_views;
    uint request_count;
    uint frame_number;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

bool is_visible(CullingView view, float3 bounds_min, float3 bounds_max) {
    for (int i = 0; i < 6; i++) {
        float3 normal = view.frustum_planes[i].xyz;
        float3 point = select(normal >= 0.0, bounds_max, bounds_min);

        if (dot(normal, point) + view.frustum_planes[i].w < 0.0) {
            return false;
        }
    }
    return true;
}

uint vertex_slice_offset(Mesh mesh, Submesh submesh, uint frames_back) {
    uint count = mesh.vertex_slice_count;
    uint slice = (push_constants.frame_number + count - frames_back % count) % count;

    return submesh.vertex_offset + slice * mesh.vertex_slice_stride;
}

uint submesh_alpha_bits(Submesh submesh) {
    return push_constants.material_buffer_device_address[submesh.material_index].flags & MATERIAL_ALPHA_MODE_BITS;
}

bool append_draw(CullRequest request, Submesh submesh, uint vertex_offset, int previous_vertex_delta, uint entity_index, uint submesh_index, uint view_mask, float sort_key) {
    uint local_command;
    InterlockedAdd(push_constants.draw_count_buffer_device_address[request.count_index], 1, local_command);
    if (local_command >= request.capacity) {
        return false;
    }

    uint slot = request.draw_offset + local_command;

    DrawIndexedIndirectCommand command;
    command.index_count = submesh.index_count;
    command.instance_count = 1;
    command.first_index = submesh.index_offset;
    command.vertex_offset = int(vertex_offset);
    command.first_instance = slot;
    push_constants.indirect_buffer_device_address[slot] = command;

    DrawData draw_data;
    draw_data.entity_index = entity_index;
    draw_data.submesh_index = submesh_index;
    draw_data.previous_vertex_delta = previous_vertex_delta;
    draw_data.cascade_mask = view_mask;
    draw_data.sort_key = sort_key;
    push_constants.draw_data_buffer_device_address[slot] = draw_data;

    return true;
}

[shader("compute")]
[numthreads(256, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    uint entity_index = global_invocation_id.x;
    if (entity_index >= push_constants.entity_count) {
        return;
    }

    Entity entity = push_constants.entity_buffer_device_address[entity_index];
    Mesh mesh = push_constants.mesh_buffer_device_address[entity.mesh_index];
    Ptr<CullingIndirectMetaStatisticsReadback> statistics = push_constants.meta_statistics_buffer_device_address;

    for (uint local_submesh_index = 0; local_submesh_index < mesh.submesh_count; local_submesh_index++) {
        uint submesh_index = mesh.submesh_offset + local_submesh_index;
        Submesh submesh = push_constants.submesh_buffer_device_address[submesh_index];
        SubmeshBounds bounds = push_constants.submesh_bounds_buffer_device_address[submesh.bounds_index];

        uint vertex_offset = vertex_slice_offset(mesh, submesh, 0u);
        int previous_vertex_delta = int(vertex_slice_offset(mesh, submesh, 1u)) - int(vertex_offset);

        float3 world_min;
        float3 world_max;
        transform_aabb(entity.transform_matrix, bounds.min, bounds.max, world_min, world_max);

        float3 world_center = (world_min + world_max) * 0.5;

        float sort_key = distance(push_constants.camera_buffer_device_address.position, world_center);
        uint alpha_bits = submesh_alpha_bits(submesh);

        uint view_mask = 0u;
        for (uint view = 0u; view < push_constants.view_count; view++) {
            if (is_visible(push_constants.culling_views_buffer_device_address[view], world_min, world_max)) {
                view_mask |= (1u << view);
            }
        }

        for (uint request_index = 0u; request_index < push_constants.request_count; request_index++) {
            CullRequest request = push_constants.cull_requests_buffer_device_address[request_index];

            if ((request.accept_mask & alpha_bits) == 0u) {
                continue;
            }

            InterlockedOr(statistics.header.written, 1);

            if (view_mask == 0u) {
                InterlockedAdd(statistics.data[request_index].submeshes_culled, 1);

                continue;
            }

            bool appended = true;

            if (push_constants.combine_views == 0u) {
                for (uint view = 0u; view < push_constants.view_count; view++) {
                    if ((view_mask & (1u << view)) != 0u) {
                        appended = append_draw(request, submesh, vertex_offset, previous_vertex_delta, entity_index, submesh_index, 1u, sort_key) && appended;
                    }
                }
            } else {
                appended = append_draw(request, submesh, vertex_offset, previous_vertex_delta, entity_index, submesh_index, view_mask, sort_key);
            }

            if (appended) {
                InterlockedAdd(statistics.data[request_index].submeshes_rendered, 1);
            } else {
                InterlockedAdd(statistics.data[request_index].submeshes_dropped, 1);
            }
        }
    }
}
