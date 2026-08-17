pub trait TaskScheduler: Send + Sync + 'static {
    fn schedule(&self, task: Box<dyn FnOnce() + Send + 'static>);
}
