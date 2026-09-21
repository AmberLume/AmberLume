use ash::vk::DeviceSize;
use gpu::ManagedBuffer;

pub struct ReadbackEntry {
    pub allocation: ManagedBuffer,
    pub frame_size: DeviceSize,

    pub snapshot: Vec<u8>,
}
