use crate::render::readback::gpu_readback::GpuReadback;
use anyhow::Result;
use ash::vk::{Buffer, BufferUsageFlags, DeviceSize};
use gpu::BufferBuilder;
use gpu::BufferInfo;
use gpu::FrameBuffer;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use gpu_allocator::MemoryLocation;
use index_allocator::FrameIndex;
use index_allocator::SliceIndex;
use render_graph::BufferResourceScope;
use render_graph::FrameContext;
use render_graph::ImageResourceScope;
use render_graph::PassGraph;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualBuffer;
use std::sync::Arc;

pub struct ReadbackSlice {
    handle: Buffer,
    offset: DeviceSize,
    mapped_ptr: *const u8,
}

impl ReadbackSlice {
    pub fn copy_target(&self) -> (Buffer, DeviceSize) {
        (self.handle, self.offset)
    }

    pub fn mapped_ptr(&self) -> *const u8 {
        self.mapped_ptr
    }
}

struct ReadbackEntry {
    readback: Arc<dyn GpuReadback>,
    offset: u32,
}

pub struct Readbacks {
    buffer: FrameBuffer<SliceBuffer<u8>>,
    total_size: DeviceSize,

    entries: Vec<ReadbackEntry>,
}

impl Readbacks {
    pub fn new(
        buffer_factory: &ManagedBufferFactory,
        readbacks: Vec<Arc<dyn GpuReadback>>,
        frame_count: u32,
    ) -> Result<Self> {
        let mut entries = Vec::with_capacity(readbacks.len());
        let mut cursor = 0;

        for readback in readbacks {
            let size = readback.size();

            entries.push(ReadbackEntry {
                readback,
                offset: cursor,
            });

            cursor += size;
        }

        let buffer = BufferBuilder::slice::<u8>(cursor.max(1))
            .per_frame(frame_count)
            .build(
                buffer_factory,
                "readbacks",
                BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuToCpu,
            )?;

        Ok(Self {
            buffer,
            total_size: cursor.max(1) as DeviceSize * frame_count as DeviceSize,

            entries,
        })
    }

    pub fn import(&self, pass_graph: &mut PassGraph) -> VirtualBuffer {
        let view = self
            .buffer
            .frame(FrameIndex::ZERO)
            .slice_at(SliceIndex::ZERO);

        pass_graph.import_buffer(
            "readbacks",
            view.handle(),
            view.offset(),
            self.total_size,
            view.device_address(),
            view.mapped_ptr(),
        )
    }

    pub fn declare(&self, declaration: &mut PassResourceDeclaration) {
        for entry in &self.entries {
            entry.readback.declare(declaration);
        }
    }

    pub fn record(
        &self,
        context: &FrameContext,
        image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
    ) {
        if self.entries.is_empty() {
            return;
        }

        for entry in &self.entries {
            let slice = self.slice(entry, context.frame_index);

            entry
                .readback
                .record(context, image_scope, buffer_scope, &slice);
        }
    }

    pub fn sync(&self, frame_index: FrameIndex) {
        for entry in &self.entries {
            let slice = self.slice(entry, frame_index);

            entry.readback.sync(&slice);
        }
    }

    fn slice(&self, entry: &ReadbackEntry, frame_index: FrameIndex) -> ReadbackSlice {
        let view = self
            .buffer
            .frame(frame_index)
            .slice_at(SliceIndex::from(entry.offset));

        ReadbackSlice {
            handle: view.handle(),
            offset: view.offset(),
            mapped_ptr: view.mapped_ptr(),
        }
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.buffer.into_managed_buffer())
    }
}
