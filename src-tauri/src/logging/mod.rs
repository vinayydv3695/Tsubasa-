// Tsubasa (翼) — Logging
// Structured logging with file rotation and in-memory ring buffer for the UI viewer.

pub mod ring_buffer;

use std::path::PathBuf;
use std::sync::Arc;

use ring_buffer::{LogEntry, LogRingBuffer};
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// A tracing Layer that writes formatted log entries into the LogRingBuffer.
struct RingBufferLayer {
    buffer: Arc<LogRingBuffer>,
}

impl<S> Layer<S> for RingBufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Extract message from event fields
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let metadata = event.metadata();
        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: metadata.level().to_string(),
            target: metadata.target().to_string(),
            message: visitor.message,
        };

        self.buffer.push(entry);
    }
}

/// Visitor that extracts the `message` field from a tracing event.
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}

/// Initialize the logging system.
/// - File logging with daily rotation
/// - Console logging for development
/// - Ring buffer for in-app log viewer
pub fn init_logging(
    log_dir: PathBuf,
    log_buffer: Option<Arc<LogRingBuffer>>,
) -> crate::error::Result<()> {
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = rolling::daily(&log_dir, "tsubasa.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard so it lives for the entire program lifetime.
    std::mem::forget(_guard);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tsubasa_lib=info,librqbit=warn"));

    let ring_layer = log_buffer.map(|buffer| RingBufferLayer { buffer });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_thread_ids(true)
                .with_ansi(false)
                .json(),
        )
        .with(ring_layer)
        .init();

    tracing::info!(
        "Tsubasa logging initialized, log dir: {}",
        log_dir.display()
    );
    Ok(())
}
