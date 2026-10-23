import "../common.slang";
import "../readback.slang";

struct DrawSortStatistics {
    uint sorted_count;
    uint unsorted_count;

    uint _pad0[2];
};

struct DrawSortStatisticsReadback {
    ReadbackHeader header;

    DrawSortStatistics data;
};

struct PushConstants {
    Ptr<DrawIndexedIndirectCommand, Access.Read> indirect_source_buffer_device_address;
    Ptr<DrawIndexedIndirectCommand> indirect_sorted_buffer_device_address;
    Ptr<uint, Access.Read> draw_count_buffer_device_address;
    Ptr<DrawData, Access.Read> draw_data_buffer_device_address;
    Ptr<DrawSortStatisticsReadback> statistics_buffer_device_address;

    uint count_index;
    uint source_offset;
    uint target_offset;
    uint capacity;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[vk::constant_id(0)] const uint GROUP_SIZE = 1024;

static const float EMPTY_SORT_KEY = -1.0;

groupshared float shared_keys[GROUP_SIZE];
groupshared uint shared_slots[GROUP_SIZE];

void copy_command(uint source_index, uint target_index) {
    push_constants.indirect_sorted_buffer_device_address[push_constants.target_offset + target_index] =
        push_constants.indirect_source_buffer_device_address[push_constants.source_offset + source_index];
}

[shader("compute")]
[numthreads(GROUP_SIZE, 1, 1)]
void main(uint3 local_invocation_id : SV_GroupThreadID) {
    uint capacity = min(GROUP_SIZE, push_constants.capacity);
    uint index = local_invocation_id.x;

    uint draw_count = min(push_constants.draw_count_buffer_device_address[push_constants.count_index], push_constants.capacity);
    uint sorted_count = min(draw_count, capacity);

    if (index == 0u) {
        Ptr<DrawSortStatisticsReadback> statistics = push_constants.statistics_buffer_device_address;

        InterlockedOr(statistics.header.written, 1);
        statistics.data.sorted_count = sorted_count;
        statistics.data.unsorted_count = draw_count - sorted_count;
    }

    shared_slots[index] = index;

    if (index < sorted_count) {
        uint slot = push_constants.indirect_source_buffer_device_address[push_constants.source_offset + index].first_instance;

        shared_keys[index] = push_constants.draw_data_buffer_device_address[slot].sort_key;
    } else {
        shared_keys[index] = EMPTY_SORT_KEY;
    }

    GroupMemoryBarrierWithGroupSync();

    for (uint span = 2u; span <= capacity; span <<= 1u) {
        for (uint stride = span >> 1u; stride > 0u; stride >>= 1u) {
            uint partner = index ^ stride;

            float own_key = shared_keys[index];
            float partner_key = shared_keys[partner];
            uint own_slot = shared_slots[index];
            uint partner_slot = shared_slots[partner];

            GroupMemoryBarrierWithGroupSync();

            bool descending = (index & span) == 0u;
            bool want_greater = descending == (index < partner);
            bool take_partner = want_greater
                ? partner_key > own_key
                : partner_key < own_key;

            shared_keys[index] = take_partner ? partner_key : own_key;
            shared_slots[index] = take_partner ? partner_slot : own_slot;

            GroupMemoryBarrierWithGroupSync();
        }
    }

    if (index < sorted_count) {
        copy_command(shared_slots[index], index);
    }

    for (uint tail = sorted_count + index; tail < draw_count; tail += capacity) {
        copy_command(tail, tail);
    }
}
