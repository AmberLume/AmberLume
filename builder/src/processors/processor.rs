use std::sync::Arc;
use anyhow::Result;
use crate::dispatcher::Dispatcher;

pub trait Processor<T> {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &T) -> Result<()>;
}
