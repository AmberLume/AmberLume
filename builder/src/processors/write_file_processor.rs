use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::Arc;
use crate::processors::processor::Processor;
use crate::build_task::WriteFileTask;
use anyhow::Result;
use tracing::info;
use crate::cache::Cache;
use crate::dispatcher::Dispatcher;

pub struct WriteFileProcessor {
    cache: Arc<Cache>,
}

impl WriteFileProcessor {
    pub fn create(
        cache: Arc<Cache>,
    ) -> Self {
        Self {
            cache,
        }
    }
}

impl Processor<WriteFileTask> for WriteFileProcessor {
    fn process(&self, _dispatcher: Arc<Dispatcher>, task: &WriteFileTask) -> Result<()> {
        info!("Writing data into {}", task.target_path.display());

        if let Some(parent) = task.target_path.parent() {
            create_dir_all(parent)?;
        }

        let mut file = File::create(&task.target_path)?;
        file.write_all(&task.data)?;

        self.cache.record_output(&task.origin_key, task.target_path.to_string_lossy().into_owned());

        Ok(())
    }
}
