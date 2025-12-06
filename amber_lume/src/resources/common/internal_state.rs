// use anyhow::Error;
// use std::sync::Arc;
//
// #[derive(Debug, Clone)]
// pub enum InternalState<T> {
//     Creating { task_id: u64 },
//     Ready { resource: Arc<T> },
//     Failed { error: Arc<Error> },
//     Evicted,
// }
