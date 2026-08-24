use gpu::FrameRegions;
use gpu::ManagedBuffer;

pub struct ReadbackEntry {
    pub allocation: ManagedBuffer,
    pub frames: FrameRegions,

    pub snapshot: Vec<u8>,
}
