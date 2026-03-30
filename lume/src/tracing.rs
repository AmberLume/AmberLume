pub struct Tracing;

impl Tracing {
    pub fn initialize() {
        tracing_subscriber::fmt()
            .with_target(true)
            .with_thread_names(true)
            // .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
            .init();
    }
}
