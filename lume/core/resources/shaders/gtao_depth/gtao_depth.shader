import "../common.slang";
import "../bindings.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint depth_descriptor_id;
    uint view_z_storage_id;
    uint width;
    uint height;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.width, push_constants.height);
    if (any(coord >= size)) {
        return;
    }

    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

    float depth = graph_textures[push_constants.depth_descriptor_id].Load(int3(coord, 0)).r;
    float view_z = (camera.near * camera.far) / (camera.near + depth * (camera.far - camera.near));

    storage_images_r16f[push_constants.view_z_storage_id][coord] = view_z;
}
