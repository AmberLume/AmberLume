use crate::cascade_statistics_gpu::CascadeStatisticsGPU;
use crate::culling_statistics_gpu::CullingIndirectRequestStatisticsGPU;
use crate::draw_sort_statistics_gpu::DrawSortStatisticsGPU;
use render_graph::HeapAllocatorStatistics;

#[derive(Clone)]
pub struct RenderStatistics {
    pub main_culling: Option<Vec<CullingIndirectRequestStatisticsGPU>>,
    pub cascade_culling: Option<Vec<CullingIndirectRequestStatisticsGPU>>,
    pub cascade_compute: Option<Vec<CascadeStatisticsGPU>>,
    pub draw_sort: Option<DrawSortStatisticsGPU>,

    pub cpu_to_gpu_allocator_statistics: HeapAllocatorStatistics,
    pub hdr_supported: bool,
}
