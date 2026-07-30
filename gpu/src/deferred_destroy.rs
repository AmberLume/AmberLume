use anyhow::Result;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

struct Entry<T> {
    item: T,
    ready_frame: u64,
}

pub struct DeferredDestroy<T> {
    queue: Mutex<VecDeque<Entry<T>>>,
    frames_in_flight: u32,
    current_frame: Arc<AtomicU64>,
    destroy: Box<dyn Fn(T) -> Result<()> + Send + Sync>,
}

impl<T> DeferredDestroy<T> {
    pub fn new(
        frames_in_flight: u32,
        current_frame: Arc<AtomicU64>,
        destroy: impl Fn(T) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            frames_in_flight,
            current_frame,
            destroy: Box::new(destroy),
        }
    }

    pub fn push(&self, item: T) {
        let ready_frame =
            self.current_frame.load(Ordering::Relaxed) + self.frames_in_flight as u64;

        self.queue.lock().push_back(Entry { item, ready_frame });
    }

    pub fn cleanup(&self) -> Result<()> {
        let current_frame = self.current_frame.load(Ordering::Relaxed);

        let mut ready_to_destroy = Vec::new();
        {
            let mut queue = self.queue.lock();

            while let Some(front) = queue.front() {
                if front.ready_frame > current_frame {
                    break;
                }

                ready_to_destroy.push(queue.pop_front().unwrap().item);
            }
        }

        for item in ready_to_destroy {
            (self.destroy)(item)?;
        }

        Ok(())
    }

    pub fn destroy_all(&self) -> Result<()> {
        let all = self
            .queue
            .lock()
            .drain(..)
            .map(|entry| entry.item)
            .collect::<Vec<_>>();

        for item in all {
            (self.destroy)(item)?;
        }

        Ok(())
    }
}
