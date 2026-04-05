use ash::vk::{Buffer, DeviceAddress};

pub struct ResourceBuffers {
    pub index_buffer_handle: Buffer,

    pub mesh_buffer: DeviceAddress,
    pub submesh_buffer: DeviceAddress,
    pub index_buffer: DeviceAddress,
    pub vertex_buffer: DeviceAddress,

    pub skeleton_buffer: DeviceAddress,
    pub skeleton_bone_buffer: DeviceAddress,

    pub animation_buffer: DeviceAddress,
    pub animation_frame_buffer: DeviceAddress,

    pub bone_transform_buffer: DeviceAddress,
    pub skinning_instance_buffer: DeviceAddress,

    pub material_buffer: DeviceAddress,
}
