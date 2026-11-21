use crate::platform_providers::tracing_message::TracingMessage;
use anyhow::{anyhow, Result};
use logger::{LogLevel, LogRecord, Logger};
use tracing::{Event, Level, Metadata, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};

pub struct TracingBridge;

impl TracingBridge {
    pub fn install() -> Result<()> {
        let subscriber = tracing_subscriber::registry().with(TracingBridge);

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|error| anyhow!("tracing bridge install: {error}"))
    }

    fn level(level: &Level) -> LogLevel {
        match *level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

impl<S: Subscriber> Layer<S> for TracingBridge {
    fn enabled(&self, metadata: &Metadata<'_>, _context: Context<'_, S>) -> bool {
        Logger::enabled(Self::level(metadata.level()))
    }

    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let metadata = event.metadata();

        let mut message = TracingMessage::new();
        event.record(&mut message);
        let text = message.into_text();

        Logger::log(&LogRecord {
            level: Self::level(metadata.level()),
            target: metadata.target(),
            message: format_args!("{text}"),
            file: metadata.file(),
            line: metadata.line(),
        });
    }
}
