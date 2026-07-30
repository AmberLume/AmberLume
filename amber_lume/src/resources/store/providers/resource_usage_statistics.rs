use gpu::IndexManagerStatistics;

pub struct ResourceUsageStatistics<S> {
    pub index: IndexManagerStatistics,
    
    pub backend: S,
}
