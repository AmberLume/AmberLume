use crate::render::render_graph::virtual_buffer::heap_allocator_statistics::HeapAllocatorStatistics;

pub struct RenderStatistics {
    pub cpu_to_gpu_allocator_statistics: HeapAllocatorStatistics,
}
