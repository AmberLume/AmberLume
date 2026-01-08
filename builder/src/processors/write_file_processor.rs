use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::Arc;
use crate::processors::processor::Processor;
use crate::build_task::WriteFileTask;
use anyhow::Result;
use tracing::info;
use crate::dispatcher::Dispatcher;

pub struct WriteFileProcessor;

impl WriteFileProcessor {
    pub fn create() -> Self {
        Self {
            
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
        
        Ok(())
    }
}
