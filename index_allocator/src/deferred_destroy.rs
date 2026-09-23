use anyhow::Result;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

struct Entry {
    release: Box<dyn FnOnce() -> Result<()> + Send>,
    ready_frame: u64,
}

pub struct DeferredDestroy {
    queue: Mutex<VecDeque<Entry>>,
    frames_in_flight: u32,
    current_frame: Arc<AtomicU64>,
}

impl DeferredDestroy {
    pub fn new(frames_in_flight: u32, current_frame: Arc<AtomicU64>) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            frames_in_flight,
            current_frame,
        }
    }

    pub fn push(&self, release: impl FnOnce() -> Result<()> + Send + 'static) {
        let ready_frame =
            self.current_frame.load(Ordering::Relaxed) + self.frames_in_flight as u64;

        self.queue.lock().push_back(Entry {
            release: Box::new(release),
            ready_frame,
        });
    }

    pub fn cleanup(&self) -> Result<()> {
        let current_frame = self.current_frame.load(Ordering::Relaxed);

        let mut ready = Vec::new();
        {
            let mut queue = self.queue.lock();

            while let Some(front) = queue.front() {
                if front.ready_frame > current_frame {
                    break;
                }

                ready.push(queue.pop_front().unwrap());
            }
        }

        Self::release(ready)
    }

    pub fn destroy_all(&self) -> Result<()> {
        let all = self.queue.lock().drain(..).collect::<Vec<_>>();

        Self::release(all)
    }

    fn release(entries: Vec<Entry>) -> Result<()> {
        let mut result = Ok(());

        for entry in entries {
            let released = (entry.release)();

            if result.is_ok() {
                result = released;
            }
        }

        result
    }
}
