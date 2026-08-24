use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use ash::vk::DeviceSize;
use index_allocator::FrameIndex;

#[derive(Clone, Copy)]
pub struct FrameRegions {
    range: BufferRange,

    frame_size: DeviceSize,
    frame_count: u32,
}

impl FrameRegions {
    pub fn create(range: BufferRange, frame_size: DeviceSize, frame_count: u32) -> Self {
        assert!(
            frame_count as DeviceSize * frame_size <= range.size,
            "FrameRegions of {} frames by {} bytes does not fit range '{}' of {} bytes",
            frame_count,
            frame_size,
            range.label,
            range.size,
        );

        Self {
            range,

            frame_size,
            frame_count,
        }
    }

    pub fn frame_size(&self) -> DeviceSize {
        self.frame_size
    }

    pub fn frame(&self, index: FrameIndex) -> BufferRange {
        assert!(
            index.value < self.frame_count,
            "FrameRegions '{}' frame {} out of bounds, count {}",
            self.range.label,
            index.value,
            self.frame_count,
        );

        self.range
            .sub(self.frame_size * index.value as DeviceSize, self.frame_size)
            .expect("FrameRegions frame within count must fit the range")
    }
}
