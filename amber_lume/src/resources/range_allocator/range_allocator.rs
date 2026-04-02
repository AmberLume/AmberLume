use parking_lot::Mutex;
use std::collections::BTreeMap;
use crate::resources::range_allocator::range_allocator_statistics::RangeAllocatorStatistics;

#[derive(Debug, Clone, Copy)]
pub struct Allocation {
    pub offset: u32,
    pub size: u32,
}

struct RangeState {
    free: BTreeMap<u32, u32>,
}

pub struct RangeAllocator {
    inner: Mutex<RangeState>,
    capacity: u32,
}

impl RangeAllocator {
    pub fn new(capacity: u32) -> Self {
        let mut free = BTreeMap::new();

        free.insert(0u32, capacity);

        Self {
            inner: Mutex::new(RangeState {
                free,
            }),
            capacity,
        }
    }

    pub fn allocate(&self, size: u32) -> Option<Allocation> {
        if size == 0 {
            return None;
        }
        let mut inner = self.inner.lock();

        let found = inner.free.iter()
            .find(|(_, free_size)| **free_size >= size)
            .map(|(start, free_size)| (start, free_size));

        let (&offset, &free_size) = found?;

        inner.free.remove(&offset);

        if free_size > size {
            inner.free.insert(offset + size, free_size - size);
        }

        Some(Allocation {
            offset,
            size,
        })
    }

    pub fn release(&self, allocation: Allocation) {
        let mut inner = self.inner.lock();

        let mut offset = allocation.offset;
        let mut size = allocation.size;

        if let Some(&right_size) = inner.free.get(&(offset + size)) {
            inner.free.remove(&(offset + size));

            size += right_size;
        }
        if let Some((&left_offset, &left_size)) = inner.free.range(..offset).next_back() {
            if left_offset + left_size == offset {
                inner.free.remove(&left_offset);

                offset = left_offset;
                size += left_size;
            }
        }

        inner.free.insert(offset, size);
    }

    pub fn statistics(&self) -> RangeAllocatorStatistics {
        let inner = self.inner.lock();

        let free_blocks: Vec<Allocation> = inner.free
            .iter()
            .map(|(&offset, &size)| Allocation { offset, size })
            .collect();

        let free = free_blocks.iter().map(|a| a.size).sum();
        let largest_free = free_blocks.iter().map(|a| a.size).max().unwrap_or(0);
        let fragmentation = free_blocks.len().saturating_sub(1) as u32;

        RangeAllocatorStatistics {
            capacity: self.capacity,
            used: self.capacity - free,
            free,

            free_blocks,

            largest_free,
            fragmentation,
        }
    }
}
