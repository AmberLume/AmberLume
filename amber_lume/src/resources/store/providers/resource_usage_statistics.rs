use crate::resources::index::index_manager_statistics::IndexManagerStatistics;

pub struct ResourceUsageStatistics<S> {
    pub index: IndexManagerStatistics,
    
    pub backend: S,
}
