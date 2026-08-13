use ash::vk::DeviceSize;
use gpu::FrameBuffer;
use gpu::SliceBuffer;

pub struct ReadbackEntry {
    pub buffer: FrameBuffer<SliceBuffer<u8>>,
    pub frame_size: DeviceSize,
}
 