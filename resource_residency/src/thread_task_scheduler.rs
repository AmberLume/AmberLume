use crate::task_scheduler::TaskScheduler;
use std::thread::spawn;

pub struct ThreadTaskScheduler;

impl ThreadTaskScheduler {
    pub fn create() -> Self {
        Self
    }
}

impl TaskScheduler for ThreadTaskScheduler {
    fn schedule(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        spawn(task);
    }
}
