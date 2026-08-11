//! Telemetry module - metrics, logging, and tracing
//! Implements observability for CNWS

pub mod metrics;
pub mod logging;
pub mod tracing;

pub use metrics::CnwsMetrics;
pub use logging::{CnwsLogger, LogEntry, LogLevel};
pub use tracing::{CnwsTracer, SpanData, SpanGuard, SpanKind, SpanStatus, SpanEvent, TraceContext};
