//! Structured logging for CNWS
//! Implements JSON logging with trace context

use crate::error::{CnwsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::TRACE => Self::Trace,
            Level::DEBUG => Self::Debug,
            Level::INFO => Self::Info,
            Level::WARN => Self::Warn,
            Level::ERROR => Self::Error,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: u64,
    /// Log level
    pub level: LogLevel,
    /// Message
    pub message: String,
    /// Target (module path)
    pub target: String,
    /// File location
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Thread ID
    pub thread_id: String,
    /// Trace ID (if available)
    pub trace_id: Option<String>,
    /// Span ID (if available)
    pub span_id: Option<String>,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            timestamp: now,
            level,
            message: message.into(),
            target: target.into(),
            file: None,
            line: None,
            thread_id: format!("{:?}", std::thread::current().id()),
            trace_id: None,
            span_id: None,
            fields: HashMap::new(),
        }
    }

    /// Add field
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Set trace context
    pub fn with_trace_context(mut self, trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self.span_id = Some(span_id.into());
        self
    }
}

/// CNWS logger
pub struct CnwsLogger {
    _private: (),
}

impl CnwsLogger {
    /// Create a new logger
    pub fn init() -> Result<()> {
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(true)
            );

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| CnwsError::Internal(format!("Failed to set global subscriber: {}", e)))?;
        Ok(())
    }

    /// Create a JSON logger
    pub fn init_json() -> Result<()> {
        let layer = JsonLogLayer::new();
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| CnwsError::Internal(format!("Failed to set global subscriber: {}", e)))?;
        Ok(())
    }
}

/// JSON log layer
struct JsonLogLayer {
    entries: Arc<parking_lot::Mutex<Vec<LogEntry>>>,
}

impl JsonLogLayer {
    fn new() -> Self {
        Self {
            entries: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }
}

impl<S> Layer<S> for JsonLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        struct FieldVisitor {
            fields: HashMap<String, serde_json::Value>,
            message: String,
        }

        impl tracing::field::Visit for FieldVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = format!("{:?}", value);
                } else {
                    self.fields.insert(
                        field.name().to_string(),
                        serde_json::to_value(format!("{:?}", value))
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
            }
        }

        let mut visitor = FieldVisitor {
            fields: HashMap::new(),
            message: String::new(),
        };
        event.record(&mut visitor);

        let level = LogLevel::from(*event.metadata().level());
        let fields_value = serde_json::to_value(&visitor.fields).unwrap_or(serde_json::Value::Null);
        let entry = LogEntry::new(level, visitor.message, event.metadata().target())
            .with_field("fields", fields_value);

        self.entries.lock().push(entry);
    }
}

impl Default for CnwsLogger {
    fn default() -> Self {
        // Ignore errors if subscriber is already set (multi-init safety)
        let _ = Self::init();
        Self { _private: () }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "test message", "test_target");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "test message");
        assert_eq!(entry.target, "test_target");
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = LogEntry::new(LogLevel::Debug, "test", "target")
            .with_field("key", "value");
        assert!(entry.fields.contains_key("key"));
    }
}
