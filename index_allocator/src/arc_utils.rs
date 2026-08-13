use std::any::type_name;
use std::panic::Location;
use anyhow::{anyhow, Result};
use std::sync::Arc;

pub trait ArcUnwrapOrErr<T> {
    fn try_unwrap(self) -> Result<T>;
}

impl<T> ArcUnwrapOrErr<T> for Arc<T> {
    #[track_caller]
    fn try_unwrap(self) -> Result<T> {
        Arc::try_unwrap(self).map_err(|arc| {
            let location = Location::caller();

            anyhow!("{} still has {} references ({}:{})",
                type_name::<T>(), Arc::strong_count(&arc),
                location.file(),
                location.line(),
            )
        })
    }
}
