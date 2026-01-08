use tracing_subscriber::EnvFilter;

pub struct Tracing {}

impl Tracing {
    pub fn initialize() {
        let filter = EnvFilter::new("builder=trace");

        tracing_subscriber::fmt()
            .with_target(true)
            .with_thread_names(true)
            .with_env_filter(filter)
            .init();
    }
}
