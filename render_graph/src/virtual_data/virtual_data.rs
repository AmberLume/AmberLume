use std::marker::PhantomData;
use crate::virtual_data::data_key::DataKey;

pub struct VirtualData<T> {
    pub key: DataKey,
    marker: PhantomData<fn() -> T>,
}

impl<T> VirtualData<T> {
    pub fn new(key: DataKey) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }
}

impl<T> Clone for VirtualData<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for VirtualData<T> {}
