use std::marker::PhantomData;

pub struct VirtualReadback<T> {
    pub handle: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> VirtualReadback<T> {
    pub fn new(handle: u32) -> Self {
        Self {
            handle,
            marker: PhantomData,
        }
    }
}

impl<T> Clone for VirtualReadback<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for VirtualReadback<T> {}
