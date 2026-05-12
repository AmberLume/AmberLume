use crate::render::render_graph::resource_registry::resource_registry::ResourceRegistry;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;

impl VirtualBuffer {
    pub fn stage_slice<T>(
        self,
        resource_registry: &mut ResourceRegistry,
        allocator: &mut HeapAllocator,
        data: &[T],
    ) -> anyhow::Result<()> {
        let physical_buffer = allocator.allocate_for_slice(&data)?;
        physical_buffer.write(&data)?;

        resource_registry.rebind_buffer(
            self,
            physical_buffer.buffer,
            physical_buffer.offset,
            physical_buffer.size,
            physical_buffer.device_address,
            physical_buffer.mapped_ptr,
        );

        Ok(())
    }
}
