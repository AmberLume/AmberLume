use resource_residency::TaskScheduler;
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct ManualTaskScheduler {
    tasks: Mutex<VecDeque<Box<dyn FnOnce() + Send + 'static>>>,
}

impl ManualTaskScheduler {
    pub fn create() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
        }
    }

    pub fn pending(&self) -> usize {
        self.tasks
            .lock()
            .expect("scheduler queue must not be poisoned")
            .len()
    }

    pub fn run_pending(&self) -> u32 {
        let mut executed = 0;

        while let Some(task) = self.take_next() {
            task();

            executed += 1;
        }

        executed
    }

    fn take_next(&self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        self.tasks
            .lock()
            .expect("scheduler queue must not be poisoned")
            .pop_front()
    }
}

impl TaskScheduler for ManualTaskScheduler {
    fn schedule(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.tasks
            .lock()
            .expect("scheduler queue must not be poisoned")
            .push_back(task);
    }
}
