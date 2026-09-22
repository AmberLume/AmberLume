use ash::vk::{Buffer, DeviceSize, Extent3D, Image, ImageSubresourceLayers};
use crossbeam_channel::Sender;

pub enum TransferTask {
    Buffer {
        handle: Buffer,
        data: Vec<u8>,
        offset: DeviceSize,
    },
    Image {
        handle: Image,
        data: Vec<u8>,
        extent: Extent3D,
        subresource: ImageSubresourceLayers,
        level_count: u32,
        layer_count: u32,
    },
    Flush {
        acknowledge: Sender<()>,
    },
    Terminate,
}
