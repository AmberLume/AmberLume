import "../bindings.slang";

static const uint MAX_HIZ_MIPS = 16;

struct HiZCounter {
    uint counter;
};

struct PushConstants {
    Ptr<HiZCounter> counter_buffer_device_address;

    uint depth_descriptor_id;
    uint mip_count;
    uint width;
    uint height;

    uint storage_ids[MAX_HIZ_MIPS];
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[[vk::binding(BINDING_STORAGE_IMAGE, GLOBAL_SET)]] [[vk::image_format("unknown")]] globallycoherent RWTexture2D<float2> hiz_images[];

groupshared float2 spd_lds_a[16][16];
groupshared float2 spd_lds_b[16][16];
groupshared uint spd_counter;

float2 spd_reduce4(float2 a, float2 b, float2 c, float2 d) {
    return float2(
        max(max(a.x, b.x), max(c.x, d.x)),
        min(min(a.y, b.y), min(c.y, d.y))
    );
}

float2 spd_load_depth(int2 coord, int2 size) {
    float z = graph_textures[push_constants.depth_descriptor_id].Load(int3(clamp(coord, int2(0), size - 1), 0)).r;
    return float2(z, z);
}

float2 spd_load_level(uint level, int2 coord, int2 size) {
    return hiz_images[push_constants.storage_ids[level]][clamp(coord, int2(0), size - 1)];
}

void spd_store(uint level, int2 coord, float2 value) {
    hiz_images[push_constants.storage_ids[level]][coord] = value;
}

void spd_reduce_lds(int2 group, int2 local_id, uint local_index, uint base, uint mip_count, float2 value) {
    if (mip_count > base + 1u) {
        spd_store(base + 1u, group * 16 + local_id, value);
    }
    spd_lds_a[local_id.y][local_id.x] = value;
    GroupMemoryBarrierWithGroupSync();

    if (local_index < 64u) {
        int2 t = int2(int(local_index % 8u), int(local_index / 8u));
        float2 r = spd_reduce4(
            spd_lds_a[t.y * 2][t.x * 2], spd_lds_a[t.y * 2][t.x * 2 + 1],
            spd_lds_a[t.y * 2 + 1][t.x * 2], spd_lds_a[t.y * 2 + 1][t.x * 2 + 1]
        );
        if (mip_count > base + 2u) {
            spd_store(base + 2u, group * 8 + t, r);
        }
        spd_lds_b[t.y][t.x] = r;
    }
    GroupMemoryBarrierWithGroupSync();

    if (local_index < 16u) {
        int2 t = int2(int(local_index % 4u), int(local_index / 4u));
        float2 r = spd_reduce4(
            spd_lds_b[t.y * 2][t.x * 2], spd_lds_b[t.y * 2][t.x * 2 + 1],
            spd_lds_b[t.y * 2 + 1][t.x * 2], spd_lds_b[t.y * 2 + 1][t.x * 2 + 1]
        );
        if (mip_count > base + 3u) {
            spd_store(base + 3u, group * 4 + t, r);
        }
        spd_lds_a[t.y][t.x] = r;
    }
    GroupMemoryBarrierWithGroupSync();

    if (local_index < 4u) {
        int2 t = int2(int(local_index % 2u), int(local_index / 2u));
        float2 r = spd_reduce4(
            spd_lds_a[t.y * 2][t.x * 2], spd_lds_a[t.y * 2][t.x * 2 + 1],
            spd_lds_a[t.y * 2 + 1][t.x * 2], spd_lds_a[t.y * 2 + 1][t.x * 2 + 1]
        );
        if (mip_count > base + 4u) {
            spd_store(base + 4u, group * 2 + t, r);
        }
        spd_lds_b[t.y][t.x] = r;
    }
    GroupMemoryBarrierWithGroupSync();

    if (local_index == 0u) {
        float2 r = spd_reduce4(spd_lds_b[0][0], spd_lds_b[0][1], spd_lds_b[1][0], spd_lds_b[1][1]);
        if (mip_count > base + 5u) {
            spd_store(base + 5u, group, r);
        }
    }
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 group_id : SV_GroupID, uint3 local_invocation_id : SV_GroupThreadID, uint local_invocation_index : SV_GroupIndex) {
    int2 group = int2(group_id.xy);
    int2 local_id = int2(local_invocation_id.xy);
    int2 depth_size = int2(push_constants.width, push_constants.height);
    uint mip_count = push_constants.mip_count;

    float2 q[2][2];
    for (int sy = 0; sy < 2; ++sy) {
        for (int sx = 0; sx < 2; ++sx) {
            int2 level_0_coord = group * 32 + local_id * 2 + int2(sx, sy);
            int2 depth_coord = level_0_coord * 2;
            float2 r = spd_reduce4(
                spd_load_depth(depth_coord, depth_size),
                spd_load_depth(depth_coord + int2(1, 0), depth_size),
                spd_load_depth(depth_coord + int2(0, 1), depth_size),
                spd_load_depth(depth_coord + int2(1, 1), depth_size)
            );
            q[sy][sx] = r;
            spd_store(0u, level_0_coord, r);
        }
    }

    float2 r1 = spd_reduce4(q[0][0], q[0][1], q[1][0], q[1][1]);
    spd_reduce_lds(group, local_id, local_invocation_index, 0u, mip_count, r1);

    if (mip_count <= 6u) {
        return;
    }

    if (local_invocation_index == 0u) {
        DeviceMemoryBarrier();
        InterlockedAdd(push_constants.counter_buffer_device_address.counter, 1u, spd_counter);
    }
    GroupMemoryBarrierWithGroupSync();

    uint3 group_count = WorkgroupCount();
    if (spd_counter != group_count.x * group_count.y - 1u) {
        return;
    }

    if (local_invocation_index == 0u) {
        push_constants.counter_buffer_device_address.counter = 0u;
    }
    DeviceMemoryBarrier();

    int2 level_5_size = max(int2(1), depth_size >> 6);
    float2 qb[2][2];
    for (int sy = 0; sy < 2; ++sy) {
        for (int sx = 0; sx < 2; ++sx) {
            int2 level_6_coord = local_id * 2 + int2(sx, sy);
            int2 source_coord = level_6_coord * 2;
            float2 r = spd_reduce4(
                spd_load_level(5u, source_coord, level_5_size),
                spd_load_level(5u, source_coord + int2(1, 0), level_5_size),
                spd_load_level(5u, source_coord + int2(0, 1), level_5_size),
                spd_load_level(5u, source_coord + int2(1, 1), level_5_size)
            );
            qb[sy][sx] = r;
            if (mip_count > 6u) {
                spd_store(6u, level_6_coord, r);
            }
        }
    }

    float2 rb1 = spd_reduce4(qb[0][0], qb[0][1], qb[1][0], qb[1][1]);
    spd_reduce_lds(int2(0), local_id, local_invocation_index, 6u, mip_count, rb1);
}
